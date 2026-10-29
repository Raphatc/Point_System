use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use tauri::{State, command};

use crate::infrastructure::db::AppState;
use crate::domain::{AppError, SaleInstallment};

#[derive(Debug, Serialize)]
pub struct CashStatus {
    pub is_open: bool,
    pub session_id: Option<i64>,
    pub opened_at: Option<String>,
    pub opening_amount: i64,
    pub expected_cash: i64,
    pub pix_total: i64,
    pub card_total: i64,
}

#[derive(Debug, Serialize)]
pub struct CashClosing {
    pub expected_amount: i64,
    pub counted_amount: i64,
    pub difference: i64,
}

#[derive(Debug, Serialize)]
pub struct Receivable {
    pub sale_id: i64,
    pub client_name: String,
    pub total_amount: i64,
    pub received_amount: i64,
    pub balance: i64,
    pub due_date: Option<String>,
    pub status: String,
    pub installments: Vec<SaleInstallment>,
}

fn validate_payment_method(method: &str) -> Result<(), String> {
    match method {
        "dinheiro" | "pix" | "cartao" | "cartao_credito" | "cartao_debito" => Ok(()),
        _ => Err("Forma de pagamento inválida".to_string()),
    }
}

pub(crate) fn open_session(conn: &Connection, opening_amount: i64) -> Result<i64, String> {
    if opening_amount < 0 {
        return Err("O valor de abertura não pode ser negativo".to_string());
    }

    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let open_session: Option<i64> = transaction
        .query_row(
            "SELECT id FROM cash_sessions WHERE status = 'open' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    if open_session.is_some() {
        return Err("Já existe um caixa aberto".to_string());
    }

    transaction
        .execute(
            "INSERT INTO cash_sessions (opening_amount, status) VALUES (?1, 'open')",
            [opening_amount],
        )
        .map_err(|error| error.to_string())?;
    let session_id = transaction.last_insert_rowid();
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(session_id)
}

pub(crate) fn current_session_id(conn: &Connection) -> Result<i64, String> {
    conn.query_row(
        "SELECT id FROM cash_sessions WHERE status = 'open' LIMIT 1",
        [],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "Abra o caixa antes de registrar esta operação".to_string())
}

pub(crate) fn register_movement(
    transaction: &Transaction<'_>,
    session_id: i64,
    movement_type: &str,
    amount: i64,
    method: &str,
    description: &str,
    is_demo: bool,
) -> Result<(), String> {
    transaction
        .execute(
            "INSERT INTO cash_movements (cash_session_id, movement_type, amount, method, description, is_demo)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![session_id, movement_type, amount, method, description, i64::from(is_demo)],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn record_manual_movement(
    conn: &Connection,
    movement_type: &str,
    amount: i64,
    method: &str,
    description: &str,
) -> Result<(), String> {
    if movement_type != "entrada" && movement_type != "saida" {
        return Err("Tipo de movimento inválido".to_string());
    }
    if amount <= 0 {
        return Err("O valor do movimento deve ser maior que zero".to_string());
    }
    if description.trim().is_empty() {
        return Err("Informe a descrição do movimento".to_string());
    }
    validate_payment_method(method)?;

    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let session_id = current_session_id(&transaction)?;
    let normalized_method = if method.starts_with("cartao_") {
        "cartao"
    } else {
        method
    };
    register_movement(
        &transaction,
        session_id,
        movement_type,
        amount,
        normalized_method,
        description,
        false,
    )?;
    transaction.commit().map_err(|error| error.to_string())
}

fn close_session(conn: &Connection, counted_amount: i64) -> Result<CashClosing, String> {
    if counted_amount < 0 {
        return Err("O valor contado não pode ser negativo".to_string());
    }

    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let session: Option<(i64, i64)> = transaction
        .query_row(
            "SELECT id, opening_amount FROM cash_sessions WHERE status = 'open' LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let (session_id, opening_amount) =
        session.ok_or_else(|| "Não há caixa aberto para fechar".to_string())?;

    let cash_movements: i64 = transaction
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN movement_type = 'entrada' THEN amount ELSE -amount END), 0)
             FROM cash_movements WHERE cash_session_id = ?1 AND method = 'dinheiro' AND is_demo = 0",
            [session_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    let expected_amount = opening_amount
        .checked_add(cash_movements)
        .ok_or_else(|| "Valor esperado excede o limite permitido".to_string())?;
    if expected_amount < 0 {
        return Err("O caixa não pode ficar com saldo esperado negativo".to_string());
    }
    let difference = counted_amount - expected_amount;

    transaction
        .execute(
            "UPDATE cash_sessions
             SET closed_at = CURRENT_TIMESTAMP, expected_amount = ?1, counted_amount = ?2,
                 difference = ?3, status = 'closed'
             WHERE id = ?4 AND status = 'open'",
            params![expected_amount, counted_amount, difference, session_id],
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;

    Ok(CashClosing {
        expected_amount,
        counted_amount,
        difference,
    })
}

fn receive_payment(
    conn: &Connection,
    sale_id: i64,
    installment_id: Option<i64>,
    amount: i64,
    method: &str,
) -> Result<i64, String> {
    if amount <= 0 {
        return Err("O valor recebido deve ser maior que zero".to_string());
    }
    validate_payment_method(method)?;

    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let session_id = current_session_id(&transaction)?;
    let (total_amount, is_credit, status, is_demo): (i64, i64, String, bool) = transaction
        .query_row(
            "SELECT total_amount, is_credit, status, is_demo FROM sales WHERE id = ?1",
            [sale_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Venda não encontrada".to_string())?;
    if is_credit != 1 || status != "open" || is_demo {
        return Err("A venda não possui saldo em aberto".to_string());
    }

    let balance = if let Some(installment_id) = installment_id {
        let installment_amount: i64 = transaction
            .query_row(
                "SELECT amount FROM sale_installments WHERE id = ?1 AND sale_id = ?2 AND status = 'open'",
                params![installment_id, sale_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Parcela não encontrada ou já quitada".to_string())?;
        let received: i64 = transaction
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM receivable_payments WHERE installment_id = ?1",
                [installment_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        installment_amount - received
    } else {
        let installment_count: i64 = transaction
            .query_row(
                "SELECT COUNT(*) FROM sale_installments WHERE sale_id = ?1",
                [sale_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if installment_count > 0 {
            return Err("Selecione a parcela que deseja receber".to_string());
        }
        let received: i64 = transaction
            .query_row(
                "SELECT COALESCE(SUM(amount), 0) FROM receivable_payments WHERE sale_id = ?1",
                [sale_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        total_amount - received
    };
    if amount > balance {
        return Err("O recebimento não pode exceder o saldo da venda".to_string());
    }

    let stored_method = if method.starts_with("cartao_") {
        "cartao"
    } else {
        method
    };
    let card_type = match method {
        "cartao_credito" => Some("credito"),
        "cartao_debito" => Some("debito"),
        _ => None,
    };
    transaction
        .execute(
            "INSERT INTO receivable_payments (sale_id, installment_id, cash_session_id, amount, method, card_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![sale_id, installment_id, session_id, amount, stored_method, card_type],
        )
        .map_err(|error| error.to_string())?;
    let movement_method = if method.starts_with("cartao_") {
        "cartao"
    } else {
        method
    };
    register_movement(
        &transaction,
        session_id,
        "entrada",
        amount,
        movement_method,
        &format!("Recebimento da venda #{sale_id}"),
        false,
    )?;

    if let Some(installment_id) = installment_id {
        if amount == balance {
            transaction
                .execute(
                    "UPDATE sale_installments SET status = 'paid' WHERE id = ?1",
                    [installment_id],
                )
                .map_err(|error| error.to_string())?;
        }
        let open_installments: i64 = transaction
            .query_row(
                "SELECT COUNT(*) FROM sale_installments WHERE sale_id = ?1 AND status = 'open'",
                [sale_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if open_installments == 0 {
            transaction
                .execute("UPDATE sales SET status = 'paid' WHERE id = ?1", [sale_id])
                .map_err(|error| error.to_string())?;
        }
    } else if amount == balance {
        transaction
            .execute("UPDATE sales SET status = 'paid' WHERE id = ?1", [sale_id])
            .map_err(|error| error.to_string())?;
    }

    transaction.commit().map_err(|error| error.to_string())?;
    Ok(balance - amount)
}

#[command]
pub fn open_cash_session(opening_amount: i64, state: State<AppState>) -> Result<i64, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    Ok(open_session(&conn, opening_amount)?)
}

#[command]
pub fn get_cash_status(state: State<AppState>) -> Result<CashStatus, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let session: Option<(i64, String, i64)> = conn
        .query_row(
            "SELECT id, opened_at, opening_amount FROM cash_sessions WHERE status = 'open' LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let Some((session_id, opened_at, opening_amount)) = session else {
        return Ok(CashStatus {
            is_open: false,
            session_id: None,
            opened_at: None,
            opening_amount: 0,
            expected_cash: 0,
            pix_total: 0,
            card_total: 0,
        });
    };

    let (cash_movements, pix_total, card_total): (i64, i64, i64) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN method = 'dinheiro' AND movement_type = 'entrada' THEN amount
                                  WHEN method = 'dinheiro' AND movement_type = 'saida' THEN -amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN method = 'pix' AND movement_type = 'entrada' THEN amount
                                  WHEN method = 'pix' AND movement_type = 'saida' THEN -amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN method = 'cartao' AND movement_type = 'entrada' THEN amount
                                  WHEN method = 'cartao' AND movement_type = 'saida' THEN -amount ELSE 0 END), 0)
             FROM cash_movements WHERE cash_session_id = ?1 AND is_demo = 0",
            [session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| error.to_string())?;

    let expected_cash = opening_amount
        .checked_add(cash_movements)
        .ok_or_else(|| "Valor esperado excede o limite permitido".to_string())?;
    Ok(CashStatus {
        is_open: true,
        session_id: Some(session_id),
        opened_at: Some(opened_at),
        opening_amount,
        expected_cash,
        pix_total,
        card_total,
    })
}

#[command]
pub fn register_cash_movement(
    movement_type: String,
    amount: i64,
    method: String,
    description: String,
    state: State<AppState>,
) -> Result<(), AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    Ok(record_manual_movement(&conn, &movement_type, amount, &method, &description)?)
}

#[command]
pub fn close_cash_session(
    counted_amount: i64,
    state: State<AppState>,
) -> Result<CashClosing, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    Ok(close_session(&conn, counted_amount)?)
}

#[command]
pub fn list_receivables(state: State<AppState>) -> Result<Vec<Receivable>, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut statement = conn
        .prepare(
            "SELECT s.id, COALESCE(c.name, 'Cliente não informado'), s.total_amount,
                    COALESCE(SUM(rp.amount), 0), s.due_date, s.status
             FROM sales s
             LEFT JOIN clients c ON c.id = s.client_id
             LEFT JOIN receivable_payments rp ON rp.sale_id = s.id
             WHERE s.is_credit = 1 AND s.is_demo = 0
             GROUP BY s.id
             HAVING s.total_amount - COALESCE(SUM(rp.amount), 0) > 0
             ORDER BY s.due_date ASC, s.id ASC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            let total_amount: i64 = row.get(2)?;
            let received_amount: i64 = row.get(3)?;
            Ok(Receivable {
                sale_id: row.get(0)?,
                client_name: row.get(1)?,
                total_amount,
                received_amount,
                balance: total_amount - received_amount,
                due_date: row.get(4)?,
                status: row.get(5)?,
                installments: Vec::new(),
            })
        })
        .map_err(|error| error.to_string())?;
    let mut receivables = rows
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?;
    drop(statement);

    for receivable in &mut receivables {
        let mut installment_statement = conn
            .prepare(
                "SELECT si.id, si.installment_number, si.amount,
                        COALESCE(SUM(rp.amount), 0), si.due_date, si.status
                 FROM sale_installments si
                 LEFT JOIN receivable_payments rp ON rp.installment_id = si.id
                 WHERE si.sale_id = ?1
                 GROUP BY si.id
                 ORDER BY si.installment_number",
            )
            .map_err(|error| error.to_string())?;
        receivable.installments = installment_statement
            .query_map([receivable.sale_id], |row| {
                let amount: i64 = row.get(2)?;
                let received_amount: i64 = row.get(3)?;
                Ok(SaleInstallment {
                    id: Some(row.get(0)?),
                    installment_number: row.get(1)?,
                    amount,
                    received_amount,
                    balance: amount - received_amount,
                    due_date: row.get(4)?,
                    status: row.get(5)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        if receivable.installments.is_empty()
            && let Some(due_date) = &receivable.due_date
        {
                receivable.installments.push(SaleInstallment {
                    id: None,
                    installment_number: 1,
                    amount: receivable.total_amount,
                    received_amount: receivable.received_amount,
                    balance: receivable.balance,
                    due_date: due_date.clone(),
                    status: receivable.status.clone(),
                });
            }
        }
    Ok(receivables)
}

#[command]
pub fn receive_receivable_payment(
    sale_id: i64,
    installment_id: Option<i64>,
    amount: i64,
    method: String,
    state: State<AppState>,
) -> Result<i64, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    Ok(receive_payment(&conn, sale_id, installment_id, amount, &method)?)
}

#[cfg(test)]
mod tests {
    use super::{close_session, open_session, receive_payment, record_manual_movement};
    use crate::infrastructure::db::initialize_db;
    use rusqlite::Connection;

    fn prepared_db() -> Connection {
        let conn = Connection::open_in_memory().expect("banco em memória");
        initialize_db(&conn).expect("schema criado");
        conn.execute("INSERT INTO clients (name) VALUES ('Cliente de teste')", [])
            .expect("cliente criado");
        conn.execute(
            "INSERT INTO sales (client_id, total_amount, total_cost, payment_method, is_credit, status, due_date)
             VALUES (1, 10000, 6000, 'dinheiro', 1, 'open', '2026-12-01')",
            [],
        )
        .expect("conta a receber criada");
        conn
    }

    #[test]
    fn recebimentos_parciais_atualizam_saldo_e_caixa_sem_exceder_a_divida() {
        let conn = prepared_db();
        open_session(&conn, 5000).expect("caixa aberto");

        let remaining =
            receive_payment(&conn, 1, None, 4000, "dinheiro").expect("primeiro recebimento");
        assert_eq!(remaining, 6000);
        assert!(receive_payment(&conn, 1, None, 7000, "pix").is_err());
        assert_eq!(
            receive_payment(&conn, 1, None, 6000, "pix").expect("quitação"),
            0
        );

        let status: String = conn
            .query_row("SELECT status FROM sales WHERE id = 1", [], |row| {
                row.get(0)
            })
            .expect("status da venda");
        let payment_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM receivable_payments WHERE sale_id = 1",
                [],
                |row| row.get(0),
            )
            .expect("quantidade de pagamentos");
        assert_eq!(status, "paid");
        assert_eq!(payment_count, 2);
    }

    #[test]
    fn fechamento_compara_saldo_fisico_com_movimentos_em_dinheiro() {
        let conn = prepared_db();
        open_session(&conn, 10000).expect("caixa aberto");
        record_manual_movement(&conn, "entrada", 2500, "dinheiro", "Suprimento")
            .expect("entrada registrada");
        record_manual_movement(&conn, "saida", 500, "dinheiro", "Despesa")
            .expect("saída registrada");

        let closing = close_session(&conn, 12000).expect("caixa fechado");
        assert_eq!(closing.expected_amount, 12000);
        assert_eq!(closing.difference, 0);
        assert!(close_session(&conn, 12000).is_err());
    }

    #[test]
    fn pagamentos_por_parcela_quitam_venda_e_classificam_cartao_no_caixa() {
        let conn = prepared_db();
        open_session(&conn, 1000).expect("caixa aberto");
        conn.execute(
            "INSERT INTO sale_installments (sale_id, installment_number, amount, due_date)
             VALUES (1, 1, 5000, '2026-12-01'), (1, 2, 5000, '2027-01-01')",
            [],
        )
        .expect("parcelas criadas");
        let first_installment_id: i64 = conn
            .query_row(
                "SELECT id FROM sale_installments WHERE sale_id = 1 AND installment_number = 1",
                [],
                |row| row.get(0),
            )
            .expect("primeira parcela consultada");
        let second_installment_id: i64 = conn
            .query_row(
                "SELECT id FROM sale_installments WHERE sale_id = 1 AND installment_number = 2",
                [],
                |row| row.get(0),
            )
            .expect("segunda parcela consultada");

        assert_eq!(
            receive_payment(&conn, 1, Some(first_installment_id), 2000, "pix")
                .expect("recebimento parcial"),
            3000
        );
        assert!(receive_payment(&conn, 1, Some(first_installment_id), 4000, "pix").is_err());
        assert_eq!(
            receive_payment(&conn, 1, Some(first_installment_id), 3000, "pix")
                .expect("primeira parcela quitada"),
            0
        );
        assert_eq!(
            receive_payment(
                &conn,
                1,
                Some(second_installment_id),
                5000,
                "cartao_credito"
            )
            .expect("segunda parcela quitada"),
            0
        );

        let paid_installments: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sale_installments WHERE sale_id = 1 AND status = 'paid'",
                [],
                |row| row.get(0),
            )
            .expect("parcelas pagas contadas");
        let card_movements: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cash_movements WHERE method = 'cartao'",
                [],
                |row| row.get(0),
            )
            .expect("movimento de cartão consultado");
        let sale_status: String = conn
            .query_row("SELECT status FROM sales WHERE id = 1", [], |row| {
                row.get(0)
            })
            .expect("situação da venda consultada");
        assert_eq!(paid_installments, 2);
        assert_eq!(card_movements, 1);
        assert_eq!(sale_status, "paid");
    }

    #[test]
    fn recebimentos_parciais_sao_aplicados_a_parcela_e_quitam_a_venda_ao_final() {
        let conn = prepared_db();
        open_session(&conn, 1000).expect("caixa aberto");
        conn.execute(
            "INSERT INTO sale_installments (sale_id, installment_number, amount, due_date)
             VALUES (1, 1, 5000, '2026-12-01'), (1, 2, 5000, '2027-01-01')",
            [],
        )
        .expect("parcelas criadas");
        let first_installment_id: i64 = conn
            .query_row(
                "SELECT id FROM sale_installments WHERE sale_id = 1 AND installment_number = 1",
                [],
                |row| row.get(0),
            )
            .expect("primeira parcela consultada");
        let second_installment_id: i64 = conn
            .query_row(
                "SELECT id FROM sale_installments WHERE sale_id = 1 AND installment_number = 2",
                [],
                |row| row.get(0),
            )
            .expect("segunda parcela consultada");

        assert_eq!(
            receive_payment(&conn, 1, Some(first_installment_id), 2000, "pix")
                .expect("recebimento parcial"),
            3000
        );
        assert!(receive_payment(&conn, 1, Some(first_installment_id), 4000, "pix").is_err());
        assert_eq!(
            receive_payment(&conn, 1, Some(first_installment_id), 3000, "pix")
                .expect("quitação da primeira parcela"),
            0
        );
        assert_eq!(
            receive_payment(
                &conn,
                1,
                Some(second_installment_id),
                5000,
                "cartao_credito"
            )
            .expect("quitação da segunda parcela"),
            0
        );

        let paid_installments: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sale_installments WHERE sale_id = 1 AND status = 'paid'",
                [],
                |row| row.get(0),
            )
            .expect("parcelas quitadas consultadas");
        let card_movements: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM cash_movements WHERE method = 'cartao'",
                [],
                |row| row.get(0),
            )
            .expect("movimentos de cartão consultados");
        let sale_status: String = conn
            .query_row("SELECT status FROM sales WHERE id = 1", [], |row| {
                row.get(0)
            })
            .expect("status da venda consultado");
        assert_eq!(paid_installments, 2);
        assert_eq!(card_movements, 1);
        assert_eq!(sale_status, "paid");
    }
}
