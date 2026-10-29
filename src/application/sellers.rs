use rusqlite::params;
use tauri::{State, command};

use crate::infrastructure::db::AppState;
use crate::domain::{AppError, Seller, SellerInput};

#[command]
pub fn list_sellers(state: State<AppState>) -> Result<Vec<Seller>, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut statement = conn
        .prepare("SELECT id, name, active FROM sellers WHERE active = 1 ORDER BY name ASC")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(Seller {
                id: row.get(0)?,
                name: row.get(1)?,
                active: row.get(2)?,
            })
        })
        .map_err(|error| error.to_string())?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?)
}

#[command]
pub fn create_seller(seller: SellerInput, state: State<AppState>) -> Result<Seller, AppError> {
    let name = seller.name.trim();
    if name.is_empty() {
        return Err(AppError::new("SELLER_NAME_REQUIRED", "Informe o nome do vendedor"));
    }

    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO sellers (name, active) VALUES (?1, 1)",
        params![name],
    )
    .map_err(|error| error.to_string())?;

    Ok(Seller {
        id: conn.last_insert_rowid(),
        name: name.to_string(),
        active: true,
    })
}
