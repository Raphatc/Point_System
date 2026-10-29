mod application;
mod domain;
mod infrastructure;
mod presentation;

use tauri::{Builder, generate_handler};

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
    create_product, get_dashboard_data, get_monthly_revenue, list_expiring_products, list_products,
};

fn main() {
    tracing::info!("Iniciando o Sistema Distribuidora");
    let conn = connect().expect("Não foi possível abrir o banco SQLite");
    tracing::info!("Banco SQLite aberto com sucesso");
    initialize_db(&conn).expect("Não foi possível inicializar as tabelas do banco");
    tracing::info!("Inicialização do banco concluída");

    Builder::default()
        .manage(AppState::new(conn))
        .invoke_handler(generate_handler![
            get_dashboard_data,
            list_products,
            list_clients,
            create_client,
            create_product,
            add_stock,
            create_sale,
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
