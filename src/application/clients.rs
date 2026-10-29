use rusqlite::params;
use tauri::{State, command};

use crate::infrastructure::db::AppState;
use crate::domain::{AppError, Client, ClientInput};

#[command]
pub fn list_clients(state: State<AppState>) -> Result<Vec<Client>, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut statement = conn
        .prepare("SELECT id, name, COALESCE(document, ''), COALESCE(phone, ''), is_demo, created_at FROM clients ORDER BY name ASC")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(Client {
                id: row.get(0)?,
                name: row.get(1)?,
                document: row.get(2)?,
                phone: row.get(3)?,
                is_demo: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|error| error.to_string())?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|error| error.to_string())?)
}

#[command]
pub fn create_client(client: ClientInput, state: State<AppState>) -> Result<Client, AppError> {
    let name = client.name.trim();
    if name.is_empty() {
        return Err(AppError::new("CLIENT_NAME_REQUIRED", "Informe o nome do cliente"));
    }

    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    conn.execute(
        "INSERT INTO clients (name, document, phone, is_demo) VALUES (?1, ?2, ?3, 0)",
        params![name, "", ""],
    )
    .map_err(|error| error.to_string())?;
    let id = conn.last_insert_rowid();
    let created_at = conn
        .query_row(
            "SELECT created_at FROM clients WHERE id = ?1",
            [id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    Ok(Client {
        id,
        name: name.to_string(),
        document: String::new(),
        phone: String::new(),
        is_demo: false,
        created_at,
    })
}
