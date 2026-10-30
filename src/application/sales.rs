use std::collections::HashMap;

use chrono::{Months, NaiveDate};
use rusqlite::{Transaction, params};
use tauri::{State, command};

use crate::infrastructure::db::AppState;
use crate::domain::{AppError, SaleInput, SalePaymentInput};
use crate::application::finance::{current_session_id, register_movement};

fn validate_payment_method(method: &str) -> Result<(), String> {
    match method {
        "dinheiro" | "pix" | "cartao" | "cartao_credito" | "cartao_debito" => Ok(()),
        _ => Err("Forma de pagamento inválida".to_string()),
    }
}

fn movement_method(method: &str) -> &str {
    if method.starts_with("cartao_") {
        "cartao"
    } else {
        method
    }
}

fn create_sale_record(transaction: &Transaction<'_>, sale: &SaleInput) -> Result<i64, String> {
    if sale.items.is_empty() {
        return Err("A venda precisa ter pelo menos um item".to_string());
    }
    if sale.discount < 0 || sale.surcharge < 0 {
        return Err("Desconto e acréscimo não podem ser negativos".to_string());
    }

    let seller_id = sale
        .seller_id
        .ok_or_else(|| "Selecione o vendedor responsável pela venda".to_string())?;
    let seller_is_active: bool = transaction
        .query_row(
            "SELECT active FROM sellers WHERE id = ?1",
            [seller_id],
            |row| row.get(0),
        )
        .map_err(|_| "Vendedor não encontrado".to_string())?;
    if !seller_is_active {
        return Err("O vendedor selecionado está inativo".to_string());
    }

    let session_id = if sale.is_credit {
        if sale.client_id.is_none() {
            return Err("Selecione um cliente para venda a prazo".to_string());
        }
        let due_date = sale
            .due_date
            .as_deref()
            .ok_or_else(|| "Informe o vencimento da venda a prazo".to_string())?;
        chrono::NaiveDate::parse_from_str(due_date, "%Y-%m-%d")
            .map_err(|_| "Data de vencimento inválida".to_string())?;
        if !matches!(sale.payment_method.as_str(), "credito" | "prazo") {
            return Err("A forma de pagamento deve ser prazo para venda a prazo".to_string());
        }
        None
    } else {
        if !matches!(
            sale.payment_method.as_str(),
            "dinheiro" | "pix" | "cartao_credito" | "cartao_debito" | "cartao"
        ) {
            return Err("Forma de pagamento inválida".to_string());
        }
        Some(current_session_id(transaction)?)
    };

    let installment_count = if sale.is_credit {
        if !(1..=36).contains(&sale.installments_count) {
            return Err("A quantidade de parcelas deve estar entre 1 e 36".to_string());
        }
        sale.installments_count
    } else {
        1
    };
    let first_due_date = if sale.is_credit {
        Some(
            NaiveDate::parse_from_str(sale.due_date.as_deref().unwrap_or_default(), "%Y-%m-%d")
                .map_err(|_| "Data de vencimento inválida".to_string())?,
        )
    } else {
        None
    };

    let mut requested = HashMap::<i64, i64>::new();
    let mut gross_amount = 0_i64;
    let mut total_cost = 0_i64;
    let mut costs = HashMap::<i64, i64>::new();
    let mut is_demo = false;

    if let Some(client_id) = sale.client_id {
        let client_is_demo: bool = transaction
            .query_row(
                "SELECT is_demo FROM clients WHERE id = ?1",
                [client_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        is_demo |= client_is_demo;
    }

    for item in &sale.items {
        if item.product_id <= 0 || item.quantity <= 0 || item.unit_price < 0 {
            return Err("Há um item de venda com quantidade ou preço inválido".to_string());
        }
        let (stock, average, product_is_demo): (i64, i64, bool) = transaction
            .query_row(
                "SELECT stock_quantity, cost_average, is_demo FROM products WHERE id = ?1",
                [item.product_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|error| error.to_string())?;
        is_demo |= product_is_demo;
        let total_quantity = requested
            .get(&item.product_id)
            .copied()
            .unwrap_or(0)
            .checked_add(item.quantity)
            .ok_or_else(|| "Quantidade total do produto excede o limite permitido".to_string())?;
        if total_quantity > stock {
            return Err(format!(
                "Estoque insuficiente para o produto {}",
                item.product_id
            ));
        }
        requested.insert(item.product_id, total_quantity);
        costs.insert(item.product_id, average);

        let line_amount = item
            .quantity
            .checked_mul(item.unit_price)
            .ok_or_else(|| "Valor da venda excede o limite permitido".to_string())?;
        let line_cost = item
            .quantity
            .checked_mul(average)
            .ok_or_else(|| "Custo da venda excede o limite permitido".to_string())?;
        gross_amount = gross_amount
            .checked_add(line_amount)
            .ok_or_else(|| "Valor total da venda excede o limite permitido".to_string())?;
        total_cost = total_cost
            .checked_add(line_cost)
            .ok_or_else(|| "Custo total da venda excede o limite permitido".to_string())?;
    }

    if gross_amount <= 0 {
        return Err("O total da venda deve ser maior que zero".to_string());
    }
    if sale.discount > gross_amount {
        return Err("O desconto não pode ser maior que o total da venda".to_string());
    }

    let total_amount = gross_amount
        .checked_sub(sale.discount)
        .and_then(|value| value.checked_add(sale.surcharge))
        .ok_or_else(|| "Valor total da venda excede o limite permitido".to_string())?;
    if total_amount <= 0 {
        return Err("O total da venda deve ser maior que zero".to_string());
    }

    let (change_amount, stored_payments, primary_method) = if sale.is_credit {
        (0_i64, Vec::<(String, i64)>::new(), sale.payment_method.clone())
    } else {
        let payments = if sale.payments.is_empty() {
            validate_payment_method(&sale.payment_method)?;
            vec![SalePaymentInput {
                method: sale.payment_method.clone(),
                amount: total_amount,
            }]
        } else {
            sale.payments.clone()
        };

        let mut paid = 0_i64;
        for payment in &payments {
            validate_payment_method(&payment.method)?;
            if payment.amount <= 0 {
                return Err("O valor de cada pagamento deve ser maior que zero".to_string());
            }
            paid = paid
                .checked_add(payment.amount)
                .ok_or_else(|| "Valor pago excede o limite permitido".to_string())?;
        }
        if paid < total_amount {
            return Err(format!(
                "Pagamento insuficiente. Restante: R$ {:.2}",
                (total_amount - paid) as f64 / 100.0
            ));
        }

        let change_amount = paid - total_amount;
        let cash_paid = payments
            .iter()
            .filter(|payment| payment.method == "dinheiro")
            .try_fold(0_i64, |sum, payment| sum.checked_add(payment.amount))
            .ok_or_else(|| "Valor em dinheiro excede o limite permitido".to_string())?;
        if change_amount > cash_paid {
            return Err("O troco só pode ser concedido em dinheiro".to_string());
        }

        let mut remaining_change = change_amount;
        let mut applied = Vec::<(String, i64)>::new();
        for payment in &payments {
            let amount = if payment.method == "dinheiro" && remaining_change > 0 {
                let deducted = remaining_change.min(payment.amount);
                remaining_change -= deducted;
                payment.amount - deducted
            } else {
                payment.amount
            };
            if amount > 0 {
                applied.push((payment.method.clone(), amount));
            }
        }
        if remaining_change > 0 {
            return Err("O troco só pode ser concedido em dinheiro".to_string());
        }
        let primary_method = if applied.len() == 1 {
            applied[0].0.clone()
        } else {
            "misto".to_string()
        };
        (change_amount, applied, primary_method)
    };

    transaction
        .execute(
            "INSERT INTO sales (client_id, seller_id, total_amount, total_cost, payment_method, is_credit, status, due_date, installments_count, discount, surcharge, change_amount, is_demo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                sale.client_id,
                seller_id,
                total_amount,
                total_cost,
                primary_method,
                i64::from(sale.is_credit),
                if sale.is_credit { "open" } else { "paid" },
                sale.due_date,
                installment_count,
                sale.discount,
                sale.surcharge,
                change_amount,
                i64::from(is_demo)
            ],
        )
        .map_err(|error| error.to_string())?;
    let sale_id = transaction.last_insert_rowid();

    if let Some(first_due_date) = first_due_date {
        let base_amount = total_amount / installment_count;
        let remainder = total_amount % installment_count;
        for installment_index in 0..installment_count {
            let installment_date = first_due_date
                .checked_add_months(Months::new(installment_index as u32))
                .ok_or_else(|| "Uma data de parcela está fora do intervalo válido".to_string())?;
            let installment_amount = base_amount + i64::from(installment_index < remainder);
            transaction
                .execute(
                    "INSERT INTO sale_installments (sale_id, installment_number, amount, due_date)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        sale_id,
                        installment_index + 1,
                        installment_amount,
                        installment_date.format("%Y-%m-%d").to_string()
                    ],
                )
                .map_err(|error| error.to_string())?;
        }
    }

    for item in &sale.items {
        let average = costs[&item.product_id];
        transaction
            .execute(
                "INSERT INTO sale_items (sale_id, product_id, quantity, unit_price, cost_price)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    sale_id,
                    item.product_id,
                    item.quantity,
                    item.unit_price,
                    average
                ],
            )
            .map_err(|error| error.to_string())?;
    }

    for (method, amount) in &stored_payments {
        transaction
            .execute(
                "INSERT INTO sale_payments (sale_id, method, amount) VALUES (?1, ?2, ?3)",
                params![sale_id, method, amount],
            )
            .map_err(|error| error.to_string())?;
    }

    for (product_id, quantity) in requested {
        let mut lots_statement = transaction
            .prepare(
                "SELECT id, quantity FROM product_lots
                 WHERE product_id = ?1 AND quantity > 0
                 ORDER BY expiration_date ASC, id ASC",
            )
            .map_err(|error| error.to_string())?;
        let lots = lots_statement
            .query_map([product_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        drop(lots_statement);

        let mut remaining = quantity;
        for (lot_id, lot_quantity) in lots {
            if remaining == 0 {
                break;
            }
            let used = remaining.min(lot_quantity);
            transaction
                .execute(
                    "UPDATE product_lots SET quantity = quantity - ?1 WHERE id = ?2",
                    params![used, lot_id],
                )
                .map_err(|error| error.to_string())?;
            remaining -= used;
        }
        if remaining > 0 {
            return Err(format!("Lotes insuficientes para o produto {product_id}"));
        }
        transaction
            .execute(
                "UPDATE products SET stock_quantity = stock_quantity - ?1 WHERE id = ?2",
                params![quantity, product_id],
            )
            .map_err(|error| error.to_string())?;
    }

    if let Some(session_id) = session_id {
        for (method, amount) in &stored_payments {
            register_movement(
                transaction,
                session_id,
                "entrada",
                *amount,
                movement_method(method),
                &format!("Venda #{sale_id}"),
                is_demo,
            )?;
        }
    }

    Ok(sale_id)
}

#[command]
pub fn create_sale(sale: SaleInput, state: State<AppState>) -> Result<i64, AppError> {
    let mut conn = state.conn.lock().map_err(|error| error.to_string())?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    let sale_id = create_sale_record(&transaction, &sale)?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(sale_id)
}

#[cfg(test)]
mod tests {
    use super::create_sale_record;
    use crate::infrastructure::db::initialize_db;
    use crate::domain::{SaleInput, SaleItemInput, SalePaymentInput};
    use crate::application::finance::open_session;
    use rusqlite::Connection;

    #[test]
    fn venda_a_vista_requer_caixa_e_registra_pagamento_com_baixa_de_estoque() {
        let mut conn = Connection::open_in_memory().expect("banco em memória");
        initialize_db(&conn).expect("schema criado");
        conn.execute(
            "INSERT INTO products (name, sku, unit_price, cost_price, cost_average, stock_quantity)
             VALUES ('Produto', 'SKU-1', 2500, 1200, 1200, 5)",
            [],
        )
        .expect("produto criado");
        let product_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO product_lots (product_id, quantity, cost_price) VALUES (?1, 5, 1200)",
            [product_id],
        )
        .expect("lote criado");
        open_session(&conn, 1000, None).expect("caixa aberto");
        conn.execute("INSERT INTO sellers (name) VALUES ('Vendedor 1')", [])
            .expect("vendedor criado");
        let seller_id = conn.last_insert_rowid();

        let sale = SaleInput {
            client_id: None,
            seller_id: Some(seller_id),
            payment_method: "pix".to_string(),
            is_credit: false,
            due_date: None,
            installments_count: 1,
            items: vec![SaleItemInput {
                product_id,
                quantity: 2,
                unit_price: 2500,
            }],
            discount: 0,
            surcharge: 0,
            payments: Vec::new(),
        };
        let transaction = conn.transaction().expect("transação iniciada");
        create_sale_record(&transaction, &sale).expect("venda criada");
        transaction.commit().expect("venda confirmada");

        let stock: i64 = conn
            .query_row(
                "SELECT stock_quantity FROM products WHERE id = ?1",
                [product_id],
                |row| row.get(0),
            )
            .expect("estoque consultado");
        let movement: (i64, String) = conn
            .query_row(
                "SELECT amount, method FROM cash_movements WHERE description = 'Venda #1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("movimento financeiro consultado");
        assert_eq!(stock, 3);
        assert_eq!(movement, (5000, "pix".to_string()));
    }

    #[test]
    fn venda_a_prazo_cria_recebivel_sem_entrada_imediata_no_caixa() {
        let mut conn = Connection::open_in_memory().expect("banco em memória");
        initialize_db(&conn).expect("schema criado");
        conn.execute(
            "INSERT INTO products (name, sku, unit_price, cost_price, cost_average, stock_quantity)
             VALUES ('Produto a prazo', 'SKU-CREDITO', 4000, 2000, 2000, 3)",
            [],
        )
        .expect("produto criado");
        let product_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO product_lots (product_id, quantity, cost_price) VALUES (?1, 3, 2000)",
            [product_id],
        )
        .expect("lote criado");
        conn.execute(
            "INSERT INTO clients (name, document, phone) VALUES ('Cliente real', '', '')",
            [],
        )
        .expect("cliente real criado");
        let client_id = conn.last_insert_rowid();
        conn.execute("INSERT INTO sellers (name) VALUES ('Vendedor prazo')", [])
            .expect("vendedor criado");
        let seller_id = conn.last_insert_rowid();

        let sale = SaleInput {
            client_id: Some(client_id),
            seller_id: Some(seller_id),
            payment_method: "credito".to_string(),
            is_credit: true,
            due_date: Some("2026-12-01".to_string()),
            installments_count: 3,
            items: vec![SaleItemInput {
                product_id,
                quantity: 1,
                unit_price: 4000,
            }],
            discount: 0,
            surcharge: 0,
            payments: Vec::new(),
        };
        let transaction = conn.transaction().expect("transação iniciada");
        let sale_id = create_sale_record(&transaction, &sale).expect("venda a prazo criada");
        transaction.commit().expect("venda confirmada");

        let receivable: (i64, String) = conn
            .query_row(
                "SELECT total_amount, status FROM sales WHERE id = ?1 AND is_credit = 1",
                [sale_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("conta a receber consultada");
        let movements: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cash_movements WHERE description = ?1",
                [format!("Venda #{sale_id}")],
                |row| row.get(0),
            )
            .expect("movimentos da venda consultados");
        assert_eq!(receivable, (4000, "open".to_string()));
        assert_eq!(movements, 0);
        let installments: Vec<(i64, i64, String)> = conn
            .prepare(
                "SELECT amount, installment_number, due_date FROM sale_installments
                 WHERE sale_id = ?1 ORDER BY installment_number",
            )
            .expect("consulta preparada")
            .query_map([sale_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .expect("parcelas consultadas")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("parcelas coletadas");
        assert_eq!(
            installments,
            vec![
                (1334, 1, "2026-12-01".to_string()),
                (1333, 2, "2027-01-01".to_string()),
                (1333, 3, "2027-02-01".to_string()),
            ]
        );
    }

    #[test]
    fn venda_com_desconto_acrescimo_pagamento_misto_e_troco() {
        let mut conn = Connection::open_in_memory().expect("banco em memória");
        initialize_db(&conn).expect("schema criado");
        conn.execute(
            "INSERT INTO products (name, sku, unit_price, cost_price, cost_average, stock_quantity)
             VALUES ('Produto misto', 'SKU-MISTO', 3000, 1500, 1500, 2)",
            [],
        )
        .expect("produto criado");
        let product_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO product_lots (product_id, quantity, cost_price) VALUES (?1, 2, 1500)",
            [product_id],
        )
        .expect("lote criado");
        open_session(&conn, 0, None).expect("caixa aberto");
        conn.execute("INSERT INTO sellers (name) VALUES ('Vendedor misto')", [])
            .expect("vendedor criado");
        let seller_id = conn.last_insert_rowid();

        let sale = SaleInput {
            client_id: None,
            seller_id: Some(seller_id),
            payment_method: "dinheiro".to_string(),
            is_credit: false,
            due_date: None,
            installments_count: 1,
            items: vec![SaleItemInput {
                product_id,
                quantity: 1,
                unit_price: 3000,
            }],
            discount: 500,
            surcharge: 200,
            payments: vec![
                SalePaymentInput {
                    method: "dinheiro".to_string(),
                    amount: 2000,
                },
                SalePaymentInput {
                    method: "pix".to_string(),
                    amount: 1000,
                },
            ],
        };
        let transaction = conn.transaction().expect("transação iniciada");
        let sale_id = create_sale_record(&transaction, &sale).expect("venda mista criada");
        transaction.commit().expect("venda confirmada");

        let totals: (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT total_amount, discount, surcharge, change_amount FROM sales WHERE id = ?1",
                [sale_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("totais consultados");
        assert_eq!(totals, (2700, 500, 200, 300));

        let movements: Vec<(String, i64)> = conn
            .prepare(
                "SELECT method, amount FROM cash_movements WHERE description = ?1 ORDER BY method",
            )
            .expect("consulta preparada")
            .query_map([format!("Venda #{sale_id}")], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("movimentos consultados")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("movimentos coletados");
        assert_eq!(
            movements,
            vec![("dinheiro".to_string(), 1700), ("pix".to_string(), 1000)]
        );

        let payments: Vec<(String, i64)> = conn
            .prepare("SELECT method, amount FROM sale_payments WHERE sale_id = ?1 ORDER BY method")
            .expect("consulta preparada")
            .query_map([sale_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("pagamentos consultados")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("pagamentos coletados");
        assert_eq!(
            payments,
            vec![("dinheiro".to_string(), 1700), ("pix".to_string(), 1000)]
        );
    }
}
