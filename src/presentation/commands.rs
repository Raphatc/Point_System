use std::path::PathBuf;
use std::time::Duration;

use rusqlite::backup::Backup;
use rusqlite::{Result, params};
use tauri::{AppHandle, Manager, State, command};
use tauri::path::BaseDirectory;

use crate::infrastructure::db::AppState;
use crate::domain::{AppError, DashboardData, ExpiringProduct, MonthlyRevenue, Product, ProductInput};

#[command]
pub fn get_dashboard_data(state: State<AppState>) -> Result<DashboardData, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;

    let faturamento: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_amount), 0) FROM sales WHERE is_demo = 0",
            (),
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    let lucro: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total_amount - total_cost), 0) FROM sales WHERE is_demo = 0",
            (),
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    let saldo_caixa: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN movement_type = 'entrada' THEN amount ELSE -amount END), 0) FROM cash_movements WHERE is_demo = 0",
            (),
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    let estoque_total: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(stock_quantity), 0) FROM products WHERE is_demo = 0",
            (),
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    let contas_receber: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(s.total_amount - COALESCE(p.received, 0)), 0)
             FROM sales s
             LEFT JOIN (
                SELECT sale_id, SUM(amount) AS received
                FROM receivable_payments GROUP BY sale_id
             ) p ON p.sale_id = s.id
             WHERE s.is_credit = 1 AND s.status = 'open' AND s.is_demo = 0",
            (),
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    let vendas_mes: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sales WHERE sale_date >= date('now','start of month') AND is_demo = 0",
            (),
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;

    let vendas_total: i64 = conn
        .query_row("SELECT COUNT(*) FROM sales WHERE is_demo = 0", (), |row| {
            row.get(0)
        })
        .map_err(|err| err.to_string())?;

    Ok(DashboardData {
        faturamento,
        lucro,
        saldo_caixa,
        estoque_total,
        contas_receber,
        vendas_mes,
        vendas_total,
    })
}

#[command]
pub fn list_products(state: State<AppState>) -> Result<Vec<Product>, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut statement = conn
        .prepare(
            "SELECT id, name, sku, category, gtin, unit_price, cost_price, stock_quantity, cost_average, is_demo, created_at FROM products ORDER BY name ASC",
        )
        .map_err(|err| err.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                sku: row.get(2)?,
                category: row.get(3)?,
                gtin: row.get(4)?,
                unit_price: row.get(5)?,
                cost_price: row.get(6)?,
                stock_quantity: row.get(7)?,
                cost_average: row.get(8)?,
                is_demo: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|err| err.to_string())?;

    Ok(rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?)
}

#[command]
pub fn create_product(product: ProductInput, state: State<AppState>) -> Result<Product, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO products (name, sku, category, gtin, unit_price, cost_price, cost_average, stock_quantity, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
        params![product.name, product.sku, product.category, product.gtin, product.unit_price, product.cost_price, product.cost_price, created_at],
    ).map_err(|err| err.to_string())?;

    let id: i64 = conn
        .query_row("SELECT last_insert_rowid()", (), |row| row.get(0))
        .map_err(|err| err.to_string())?;

    Ok(Product {
        id,
        name: product.name,
        sku: product.sku,
        category: product.category,
        gtin: product.gtin,
        unit_price: product.unit_price,
        cost_price: product.cost_price,
        stock_quantity: 0,
        cost_average: product.cost_price,
        is_demo: false,
        created_at,
    })
}

#[command]
pub fn get_monthly_revenue(state: State<AppState>) -> Result<Vec<MonthlyRevenue>, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut statement = conn
        .prepare(
            "SELECT strftime('%Y-%m', sale_date) AS month, COALESCE(SUM(total_amount), 0) AS total FROM sales WHERE is_demo = 0 GROUP BY month ORDER BY month DESC LIMIT 6",
        )
        .map_err(|err| err.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(MonthlyRevenue {
                month: row.get(0)?,
                total: row.get(1)?,
            })
        })
        .map_err(|err| err.to_string())?;

    let mut monthly = Vec::new();
    for item in rows {
        monthly.push(item.map_err(|err| err.to_string())?);
    }
    monthly.reverse();
    Ok(monthly)
}

#[command]
pub fn list_expiring_products(state: State<AppState>) -> Result<Vec<ExpiringProduct>, AppError> {
    let conn = state.conn.lock().map_err(|error| error.to_string())?;
    let mut statement = conn
        .prepare(
            "SELECT p.id, p.name, SUM(l.quantity) as quantity, CASE WHEN l.expiration_date < date('now', '+7 days') THEN 'Vencendo' WHEN l.expiration_date < date('now', '+30 days') THEN 'Próximo do vencimento' ELSE 'Em estoque' END AS label FROM product_lots l INNER JOIN products p ON p.id = l.product_id WHERE l.quantity > 0 AND p.is_demo = 0 GROUP BY p.id, p.name, label ORDER BY l.expiration_date ASC LIMIT 6",
        )
        .map_err(|err| err.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(ExpiringProduct {
                id: row.get(0)?,
                name: row.get(1)?,
                quantity: row.get(2)?,
                label: row.get(3)?,
            })
        })
        .map_err(|err| err.to_string())?;

    let mut expiring = Vec::new();
    for item in rows {
        expiring.push(item.map_err(|err| err.to_string())?);
    }
    Ok(expiring)
}

fn database_file_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .resolve("distribuidora.db", BaseDirectory::AppLocalData)
        .map_err(|err| err.to_string())?)
}

fn database_backup_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    Ok(app
        .path()
        .resolve("backups", BaseDirectory::AppLocalData)
        .map_err(|err| err.to_string())?)
}

#[command]
pub fn create_backup(app: AppHandle) -> Result<String, AppError> {
    let db_path = database_file_path(&app)?;
    let backup_dir = database_backup_dir(&app)?;
    std::fs::create_dir_all(&backup_dir).map_err(|err| err.to_string())?;

    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let backup_path = backup_dir.join(format!("distribuidora-backup-{timestamp}.db"));
    let source = rusqlite::Connection::open(&db_path).map_err(|err| err.to_string())?;
    let mut destination = rusqlite::Connection::open(&backup_path).map_err(|err| err.to_string())?;
    let backup = Backup::new(&source, &mut destination).map_err(|err| err.to_string())?;
    backup
        .run_to_completion(5, Duration::from_millis(250), None)
        .map_err(|err| err.to_string())?;

    Ok(backup_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("distribuidora.db")
        .to_string())
}

#[command]
pub fn list_backups(app: AppHandle) -> Result<Vec<String>, AppError> {
    let backup_dir = database_backup_dir(&app)?;
    let mut backups = Vec::new();
    let entries = std::fs::read_dir(&backup_dir).map_err(|err| err.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            backups.push(name.to_string());
        }
    }
    backups.sort();
    Ok(backups)
}

#[command]
pub fn restore_backup(app: AppHandle, file_name: String) -> Result<(), AppError> {
    let file_name = file_name.trim();
    if file_name.is_empty() || file_name.contains('/') || file_name.contains('\\') {
        return Err(AppError::new("INVALID_BACKUP_NAME", "Nome de backup inválido."));
    }
    let backup_dir = database_backup_dir(&app)?;
    let source_path = backup_dir.join(file_name);
    if !source_path.is_file() {
        return Err(AppError::new("BACKUP_NOT_FOUND", "Backup não encontrado."));
    }

    let source = rusqlite::Connection::open(&source_path).map_err(|err| err.to_string())?;
    let db_path = database_file_path(&app)?;
    let mut destination = rusqlite::Connection::open(&db_path).map_err(|err| err.to_string())?;
    let backup = Backup::new(&source, &mut destination).map_err(|err| err.to_string())?;
    backup
        .run_to_completion(5, Duration::from_millis(250), None)
        .map_err(|err| err.to_string())?;

    Ok(())
}
