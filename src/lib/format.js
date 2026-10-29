const currencyFormatter = new Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' })

export function formatCurrency(value) {
  const cents = Number(value ?? 0)
  const amount = Number.isFinite(cents) ? cents / 100 : 0
  return currencyFormatter.format(amount)
}
