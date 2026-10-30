import { describe, expect, it } from 'vitest'
import { formatCurrency } from './format'

describe('formatCurrency', () => {
  it('converte centavos em valor monetário BRL', () => {
    expect(formatCurrency(123456)).toContain('1.234,56')
  })

  it('formata valores zero', () => {
    expect(formatCurrency(0)).toContain('0,00')
  })

  it('retorna zero para valores inválidos', () => {
    expect(formatCurrency(null)).toContain('0,00')
  })
})
