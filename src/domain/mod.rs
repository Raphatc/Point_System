use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AppError {
    pub code: String,
    pub message: String,
}

impl AppError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::new("ERROR", message)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProductInput {
    pub id: Option<i64>,
    pub name: String,
    pub sku: String,
    pub category: String,
    pub gtin: String,
    pub unit_price: i64,
    pub cost_price: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub sku: String,
    pub category: String,
    pub gtin: String,
    pub unit_price: i64,
    pub cost_price: i64,
    pub stock_quantity: i64,
    pub cost_average: i64,
    pub is_demo: bool,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub document: String,
    pub phone: String,
    pub is_demo: bool,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientInput {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Seller {
    pub id: i64,
    pub name: String,
    pub active: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SellerInput {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct StockInput {
    pub product_id: i64,
    pub quantity: i64,
    pub cost_price: i64,
    pub expiration_date: String,
    pub supplier: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SaleItemInput {
    pub product_id: i64,
    pub quantity: i64,
    pub unit_price: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SalePaymentInput {
    pub method: String,
    pub amount: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SaleInput {
    pub client_id: Option<i64>,
    pub seller_id: Option<i64>,
    pub payment_method: String,
    pub is_credit: bool,
    pub due_date: Option<String>,
    pub installments_count: i64,
    pub items: Vec<SaleItemInput>,
    #[serde(default)]
    pub discount: i64,
    #[serde(default)]
    pub surcharge: i64,
    #[serde(default)]
    pub payments: Vec<SalePaymentInput>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SaleInstallment {
    pub id: Option<i64>,
    pub installment_number: i64,
    pub amount: i64,
    pub received_amount: i64,
    pub balance: i64,
    pub due_date: String,
    pub status: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DashboardData {
    pub faturamento: i64,
    pub lucro: i64,
    pub saldo_caixa: i64,
    pub estoque_total: i64,
    pub contas_receber: i64,
    pub vendas_mes: i64,
    pub vendas_total: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ExpiringProduct {
    pub id: i64,
    pub name: String,
    pub quantity: i64,
    pub label: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MonthlyRevenue {
    pub month: String,
    pub total: i64,
}

pub fn calculate_cost_average(
    current_quantity: i64,
    current_average: i64,
    incoming_quantity: i64,
    incoming_cost: i64,
) -> i64 {
    if current_quantity == 0 {
        return incoming_cost;
    }

    let total_value = (current_average * current_quantity) + (incoming_cost * incoming_quantity);
    let total_quantity = current_quantity + incoming_quantity;
    total_value / total_quantity
}

#[cfg(test)]
mod tests {
    use super::calculate_cost_average;

    #[test]
    fn custo_medio_ponderado_ajusta_matematicamente() {
        let average = calculate_cost_average(10, 1000, 5, 2000);
        assert_eq!(average, 1333);
    }
}
