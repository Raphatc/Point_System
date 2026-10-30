export interface Product {
  id: number
  name: string
  sku: string
  category: string
  gtin: string
  unit_price: number
  cost_price: number
  stock_quantity: number
  cost_average: number
  is_demo: boolean
  created_at: string
}

export interface Client {
  id: number
  name: string
  document: string
  phone: string
  is_demo: boolean
  created_at: string
}

export interface Seller {
  id: number
  name: string
  active: boolean
}

export interface CashStatus {
  is_open: boolean
  session_id: number | null
  opened_at: string | null
  opening_amount: number
  expected_cash: number
  pix_total: number
  card_total: number
  operator: string
}

export interface CashClosing {
  expected_amount: number
  counted_amount: number
  difference: number
}

export interface SaleInstallment {
  id: number | null
  installment_number: number
  amount: number
  received_amount: number
  balance: number
  due_date: string
  status: string
}

export interface Receivable {
  sale_id: number
  client_name: string
  total_amount: number
  received_amount: number
  balance: number
  due_date: string | null
  status: string
  installments: SaleInstallment[]
}

export interface DashboardData {
  faturamento: number
  lucro: number
  saldo_caixa: number
  estoque_total: number
  contas_receber: number
  vendas_mes: number
  vendas_total: number
}

export interface MonthlyRevenue {
  month: string
  total: number
}

export interface ExpiringProduct {
  id: number
  name: string
  quantity: number
  label: string
}
