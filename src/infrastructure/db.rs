use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, Result};

pub struct AppState {
    pub conn: Mutex<Connection>,
}

impl AppState {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
}

pub fn connect(path: &Path) -> Result<Connection> {
    Connection::open(path)
}

pub fn initialize_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sku TEXT NOT NULL UNIQUE,
            category TEXT NOT NULL DEFAULT 'Geral',
            gtin TEXT NOT NULL DEFAULT '',
            unit_price INTEGER NOT NULL DEFAULT 0,
            cost_price INTEGER NOT NULL DEFAULT 0,
            cost_average INTEGER NOT NULL DEFAULT 0,
            stock_quantity INTEGER NOT NULL DEFAULT 0,
            is_demo INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS product_lots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL,
            cost_price INTEGER NOT NULL,
            expiration_date TEXT,
            supplier TEXT,
            received_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(product_id) REFERENCES products(id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS clients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            document TEXT,
            phone TEXT,
            is_demo INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sellers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sales (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_id INTEGER,
            seller_id INTEGER REFERENCES sellers(id),
            total_amount INTEGER NOT NULL,
            total_cost INTEGER NOT NULL,
            payment_method TEXT NOT NULL,
            is_credit INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'open',
            sale_date TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            due_date TEXT,
            installments_count INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(client_id) REFERENCES clients(id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sale_installments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sale_id INTEGER NOT NULL REFERENCES sales(id),
            installment_number INTEGER NOT NULL,
            amount INTEGER NOT NULL CHECK (amount > 0),
            due_date TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'paid')),
            UNIQUE (sale_id, installment_number)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sale_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sale_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL,
            unit_price INTEGER NOT NULL,
            cost_price INTEGER NOT NULL,
            FOREIGN KEY(sale_id) REFERENCES sales(id),
            FOREIGN KEY(product_id) REFERENCES products(id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cash_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            opened_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            opening_amount INTEGER NOT NULL CHECK (opening_amount >= 0),
            closed_at TEXT,
            expected_amount INTEGER,
            counted_amount INTEGER,
            difference INTEGER,
            status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed'))
        )",
        (),
    )?;
    ensure_column(conn, "cash_sessions", "operator", "TEXT NOT NULL DEFAULT 'Operador'")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cash_movements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            movement_type TEXT NOT NULL,
            amount INTEGER NOT NULL,
            method TEXT NOT NULL,
            description TEXT,
            is_demo INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;

    ensure_column(
        conn,
        "cash_movements",
        "cash_session_id",
        "INTEGER REFERENCES cash_sessions(id)",
    )?;
    ensure_column(conn, "products", "is_demo", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "products", "gtin", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(conn, "clients", "is_demo", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "sales", "is_demo", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "sales", "seller_id", "INTEGER REFERENCES sellers(id)")?;
    ensure_column(
        conn,
        "sales",
        "installments_count",
        "INTEGER NOT NULL DEFAULT 1",
    )?;
    ensure_column(conn, "sales", "discount", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "sales", "surcharge", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(conn, "sales", "change_amount", "INTEGER NOT NULL DEFAULT 0")?;
    ensure_column(
        conn,
        "cash_movements",
        "is_demo",
        "INTEGER NOT NULL DEFAULT 0",
    )?;

    conn.execute(
        "UPDATE products SET is_demo = 1 WHERE sku = 'DEMO-001' AND name = 'Produto demo'",
        (),
    )?;
    conn.execute(
        "UPDATE clients SET is_demo = 1
         WHERE name = 'Cliente Demo' AND document = '000.000.000-00' AND phone = '(11) 99999-0000'",
        (),
    )?;
    conn.execute(
        "UPDATE sales SET is_demo = 1
         WHERE client_id IN (SELECT id FROM clients WHERE is_demo = 1)
            OR EXISTS (
                SELECT 1 FROM sale_items si
                INNER JOIN products p ON p.id = si.product_id
                WHERE si.sale_id = sales.id AND p.is_demo = 1
            )",
        (),
    )?;

    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS one_open_cash_session
            ON cash_sessions(status) WHERE status = 'open';
         CREATE UNIQUE INDEX IF NOT EXISTS products_gtin_unique
            ON products(gtin) WHERE gtin <> '';
         CREATE TABLE IF NOT EXISTS receivable_payments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sale_id INTEGER NOT NULL REFERENCES sales(id),
                installment_id INTEGER REFERENCES sale_installments(id),
            cash_session_id INTEGER NOT NULL REFERENCES cash_sessions(id),
            amount INTEGER NOT NULL CHECK (amount > 0),
            method TEXT NOT NULL CHECK (method IN ('dinheiro', 'pix', 'cartao')),
            paid_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );
         CREATE INDEX IF NOT EXISTS receivable_payments_sale_id
            ON receivable_payments(sale_id);
         CREATE TABLE IF NOT EXISTS sale_payments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sale_id INTEGER NOT NULL REFERENCES sales(id),
            method TEXT NOT NULL,
            amount INTEGER NOT NULL CHECK (amount > 0),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );
         CREATE INDEX IF NOT EXISTS sale_payments_sale_id
            ON sale_payments(sale_id);",
    )?;
    ensure_column(conn, "receivable_payments", "card_type", "TEXT")?;

    let existing_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM products", (), |row| row.get(0))?;
    if existing_count == 0 {
        conn.execute(
            "INSERT INTO products (name, sku, category, unit_price, cost_price, cost_average, stock_quantity, is_demo) VALUES (?, ?, ?, ?, ?, ?, ?, 1)",
            ("Produto demo", "DEMO-001", "Geral", 2500, 1200, 1200, 15),
        )?;

        conn.execute(
            "INSERT INTO product_lots (product_id, quantity, cost_price, expiration_date, supplier) VALUES (?, ?, ?, ?, ?)",
            (1, 15, 1200, "2030-12-31", "Fornecedor Demo"),
        )?;

        conn.execute(
            "INSERT INTO clients (name, document, phone, is_demo) VALUES (?, ?, ?, 1)",
            ("Cliente Demo", "000.000.000-00", "(11) 99999-0000"),
        )?;
    }

    Ok(())
}

fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<()> {
    let mut statement = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !columns.iter().any(|existing| existing == column) {
        conn.execute_batch(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {definition}"
        ))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::initialize_db;
    use rusqlite::Connection;

    #[test]
    fn inicializacao_e_migracao_de_caixa_sao_idempotentes() {
        let conn = Connection::open_in_memory().expect("banco em memória");
        initialize_db(&conn).expect("primeira inicialização");
        initialize_db(&conn).expect("segunda inicialização");

        let session_table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'cash_sessions'",
                [],
                |row| row.get(0),
            )
            .expect("consulta da tabela de caixa");
        let payment_table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'receivable_payments'",
                [],
                |row| row.get(0),
            )
            .expect("consulta da tabela de recebimentos");
        let demo_product_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM products WHERE is_demo = 1",
                [],
                |row| row.get(0),
            )
            .expect("consulta dos produtos demo");
        let demo_client_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM clients WHERE is_demo = 1",
                [],
                |row| row.get(0),
            )
            .expect("consulta dos clientes demo");

        assert_eq!(session_table_exists, 1);
        assert_eq!(payment_table_exists, 1);
        assert_eq!(demo_product_count, 1);
        assert_eq!(demo_client_count, 1);

        let installment_table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'sale_installments'",
                [],
                |row| row.get(0),
            )
            .expect("consulta das parcelas");
        let sellers_table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'sellers'",
                [],
                |row| row.get(0),
            )
            .expect("consulta de vendedores");
        assert_eq!(installment_table_exists, 1);
        assert_eq!(sellers_table_exists, 1);

        conn.execute(
            "INSERT INTO sales (client_id, total_amount, total_cost, payment_method, is_credit, status)
             VALUES (1, 2500, 1200, 'dinheiro', 0, 'paid')",
            [],
        )
        .expect("venda demo criada");
        let sale_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO sale_items (sale_id, product_id, quantity, unit_price, cost_price)
             VALUES (?1, 1, 1, 2500, 1200)",
            [sale_id],
        )
        .expect("item demo criado");
        initialize_db(&conn).expect("venda demo identificada");
        let sale_is_demo: bool = conn
            .query_row(
                "SELECT is_demo FROM sales WHERE id = ?1",
                [sale_id],
                |row| row.get(0),
            )
            .expect("venda demo consultada");
        assert!(sale_is_demo);
    }

    #[test]
    fn migracao_adiciona_vinculo_de_turno_sem_apagar_movimentos_legados() {
        let conn = Connection::open_in_memory().expect("banco em memória");
        conn.execute_batch(
            "CREATE TABLE cash_movements (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                movement_type TEXT NOT NULL,
                amount INTEGER NOT NULL,
                method TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            INSERT INTO cash_movements (movement_type, amount, method, description)
            VALUES ('entrada', 2500, 'dinheiro', 'Movimento legado');",
        )
        .expect("schema legado criado");

        initialize_db(&conn).expect("migração aplicada");
        let legacy_movement: (i64, Option<i64>, String) = conn
            .query_row(
                "SELECT amount, cash_session_id, description FROM cash_movements WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("movimento legado preservado");

        assert_eq!(
            legacy_movement,
            (2500, None, "Movimento legado".to_string())
        );
    }
}
