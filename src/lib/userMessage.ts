const knownErrors: Array<[RegExp, string]> = [
  [
    /Estoque insuficiente/i,
    'A quantidade informada supera o estoque disponível. Reduza a quantidade ou registre uma entrada.',
  ],
  [
    /Abra o caixa/i,
    'O caixa está fechado. Abra o caixa antes de registrar vendas à vista ou recebimentos.',
  ],
  [
    /não pode exceder o saldo|maior que o saldo/i,
    'O valor informado supera o saldo atualizado. Confira o saldo e tente novamente.',
  ],
  [
    /UNIQUE constraint failed: products\.sku/i,
    'Este SKU já está cadastrado. Pesquise o produto ou informe outro SKU.',
  ],
  [
    /UNIQUE constraint failed: clients/i,
    'Não foi possível salvar o cliente porque já existe um registro igual.',
  ],
  [
    /Cannot read properties of undefined.*invoke/i,
    'Esta operação funciona no aplicativo desktop. Abra o Sistema Distribuidora para continuar.',
  ],
  [
    /FOREIGN KEY constraint failed/i,
    'Um registro relacionado não existe mais. Atualize a tela e tente novamente.',
  ],
]

export function userMessage(error: unknown): string {
  const isObject = error !== null && typeof error === 'object'
  const rawMessage =
    isObject && typeof (error as { message?: unknown }).message === 'string'
      ? (error as { message: string }).message
      : String(error ?? '')
  const message = String(rawMessage)
    .replace(/^(Error|TypeError):\s*/i, '')
    .trim()

  if (!message) {
    return 'Não foi possível concluir. Confira os dados e tente novamente.'
  }

  for (const [pattern, friendlyMessage] of knownErrors) {
    if (pattern.test(message)) return friendlyMessage
  }

  if (/invoke|sqlite|sql|database|constraint/i.test(message)) {
    return 'Não foi possível concluir. Confira os dados e tente novamente.'
  }

  return message
}
