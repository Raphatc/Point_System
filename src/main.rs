mod application;
mod domain;
mod infrastructure;
mod presentation;

use tauri::{Builder, Manager, generate_handler};
use tauri::path::BaseDirectory;

use crate::application::{
    clients::{create_client, list_clients},
    finance::{
        close_cash_session, get_cash_status, list_receivables, open_cash_session,
        receive_receivable_payment, register_cash_movement,
    },
    inventory::add_stock,
    sales::create_sale,
    sellers::{create_seller, list_sellers},
};
use crate::infrastructure::db::{AppState, connect, initialize_db};
use crate::presentation::commands::{
    create_backup, create_product, get_dashboard_data, get_monthly_revenue, list_backups,
    list_expiring_products, list_products, restore_backup,
};

fn main() {
    tracing::info!("Iniciando o Sistema Distribuidora");
    Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let db_path = app
                .path()
                .resolve("distribuidora.db", BaseDirectory::AppLocalData)
                .expect("Não foi possível resolver o caminho do banco SQLite");
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).expect("Não foi possível criar o diretório do banco SQLite");
            }
            let conn = connect(&db_path).expect("Não foi possível abrir o banco SQLite");
            tracing::info!("Banco SQLite aberto em {}", db_path.display());
            initialize_db(&conn).expect("Não foi possível inicializar as tabelas do banco");
            tracing::info!("Inicialização do banco concluída");
            app.manage(AppState::new(conn));
            Ok(())
        })
        .invoke_handler(generate_handler![
            get_dashboard_data,
            list_products,
            list_clients,
            create_client,
            create_product,
            add_stock,
            create_sale,
            create_backup,
            list_backups,
            restore_backup,
            create_seller,
            list_sellers,
            open_cash_session,
            get_cash_status,
            register_cash_movement,
            close_cash_session,
            list_receivables,
            receive_receivable_payment,
            get_monthly_revenue,
            list_expiring_products,
        ])
        .run(tauri::generate_context!())
        .expect("Erro ao iniciar o app Tauri");
}
