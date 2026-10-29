use rusqlite::{Transaction, params};
use tauri::{State, command};

use crate::infrastructure::db::AppState;
use crate::domain::{AppError, StockInput, calculate_cost_average};

fn add_stock_entry(transaction: &Transaction<'_>, stock: &StockInput) -> Result<(), String> {
    if stock.product_id <= 0 || stock.quantity <= 0 || stock.cost_price < 0 {
        return Err("Produto, quantidade e custo informados são inválidos".to_string());
    }
    if !stock.expiration_date.is_empty()
        && chrono::NaiveDate::parse_from_str(&stock.expiration_date, "%Y-%m-%d").is_err()
    {
        return Err("Data de validade inválida".to_string());
    }

    let (current_average, current_quantity): (i64, i64) = transaction
        .query_row(
            "SELECT cost_average, stock_quantity FROM products WHERE id = ?1",
            [stock.product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| error.to_string())?;
    let total_quantity = current_quantity
        .checked_add(stock.quantity)
        .ok_or_else(|| "A quantidade total excede o limite permitido".to_string())?;
    let average = calculate_cost_average(
        current_quantity,
        current_average,
        stock.quantity,
        stock.cost_price,
    );

    transaction
        .execute(
            "INSERT INTO product_lots (product_id, quantity, cost_price, expiration_date, supplier)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                stock.product_id,
                stock.quantity,
                stock.cost_price,
                stock.expiration_date,
                stock.supplier
            ],
        )
        .map_err(|error| error.to_string())?;
    transaction
        .execute(
            "UPDATE products SET stock_quantity = ?1, cost_average = ?2, cost_price = ?3 WHERE id = ?4",
            params![total_quantity, average, stock.cost_price, stock.product_id],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[command]
pub fn add_stock(stock: StockInput, state: State<AppState>) -> Result<(), AppError> {
    let mut conn = state.conn.lock().map_err(|error| error.to_string())?;
    let transaction = conn.transaction().map_err(|error| error.to_string())?;
    add_stock_entry(&transaction, &stock)?;
    Ok(transaction.commit().map_err(|error| error.to_string())?)
}
