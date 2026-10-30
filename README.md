# Sistema Distribuidora

Sistema de gestão para distribuidora, desenvolvido como aplicativo desktop com **Tauri** (frontend **Vue 3** + **Vite** e backend **Rust** com **Tauri commands** e **SQLite** via **rusqlite**).

## Funcionalidades

- **Caixa / Vendas**: abertura e fechamento de caixa, lançamento de movimentos manuais, conferência de troco.
- **Registrar vendas**: consulta de produtos por nome/SKU/GTIN, carrinho de vendas, descontos/acréscimos, pagamentos (dinheiro, PIX, cartão, pagamento misto, troco) e vendas a prazo com parcelas.
- **Contas a receber**: controle de vendas a prazo, parcelas, recebimentos parciais e quitação.
- **Dashboard financeiro**: faturamento, lucro bruto, saldo de caixa, estoque, contas a receber e gráficos de faturamento/receitas.
- **Cadastros auxiliares**: produtos e estoque (com lotes e controle de validade), clientes e vendedores.
- **Backup**: criação de backups do banco SQLite e restauração de backups anteriores.

## Stack

- **Frontend**: Vue 3 (Composition API), TypeScript, Pinia (estado global), chart.js, Intl API (`pt-BR` / BRL).
- **Backend**: Rust, Tauri v2, rusqlite (SQLite), chrono, serde.

## Desenvolvimento

Pré-requisitos: Rust (stable, edition 2024), Node.js, e as dependências do sistema Tauri (webkit2gtk em Linux).

```bash
# Frontend
npm install
npm run dev

# Backend
cargo check
cargo test

# Build completo
npm run tauri build
```

## Testes

```bash
cargo test
```

## Estrutura

```
src/
  domain/        # Entidades e regras de negócio (inclui AppError)
  application/   # Casos de uso e comandos de aplicação
  infrastructure/# Acesso a banco (SQLite) e estado
  presentation/  # Comandos Tauri expostos à UI
  stores/        # Pinia stores (frontend)
  lib/           # Utilitários (formatação de moeda, mensagens de erro)
```

## Notas

- Valores financeiros são mantidos em **centavos** (inteiro) e exibidos via `Intl.NumberFormat('pt-BR', { style: 'currency', currency: 'BRL' })` centralizado em `src/lib/format.ts`.
- Erros são retornados como `AppError { code, message }` e mapeados para mensagens amigáveis no frontend.
- Logs estruturados via crate `tracing`.

## Banco de dados

O arquivo SQLite (`distribuidora.db`) é criado/resolvido no diretório de dados do aplicativo na OS, via `tauri::path`:

- Linux: `~/.local/share/Sistema Distribuidora/distribuidora.db`
- macOS: `~/Library/Application Support/com.distribuidora.local/distribuidora.db`
- Windows: `%APPDATA%\\com.distribuidora.local\\distribuidora.db`
