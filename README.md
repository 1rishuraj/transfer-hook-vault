# Transfer Hook Vault

A Token-2022 vault program on Solana that uses the **Transfer Hook** extension to enforce whitelisting and maintain an on-chain ledger atomically with every token movement.

## Overview

The vault issues a custom Token-2022 mint with a transfer hook attached. Every `transfer_checked` on this mint invokes the hook, which gates transfers to whitelisted users and automatically updates the ledger — no separate deposit/withdraw accounting instruction needed.

## Architecture

| Instruction | What it does |
|---|---|
| `initialize` | Creates the vault config PDA and the Token-2022 mint with TransferHook + MetadataPointer extensions |
| `add_user` | Admin whitelists a user by creating their `UserAccount` PDA |
| `remove_user` | Admin removes a user and closes their `UserAccount` PDA |
| `init_extra_acc_meta` | Initializes the extra account meta list required by Token-2022 for hook resolution |
| `withdraw` | Checks ledger balance and issues a delegate `approve` — client must follow with `transfer_checked` in the same tx |

The `transfer_hook` handles three cases on every token movement:

- **Deposit** (tokens → vault ATA): increments `user_account.amount`
- **Withdrawal** (tokens ← vault ATA): decrements `user_account.amount`
- **User-to-user**: whitelist check only

## Prerequisites

- Rust + Cargo
- Anchor CLI
- Solana CLI

## Build & Test

```bash
anchor build
anchor test
```

Tests run against `litesvm` (local SVM) — no validator needed.

## Program Structure

```
programs/transfer-hook-vault/src/
├── lib.rs
├── constants.rs
├── error.rs
├── state/
│   ├── vault.rs
│   └── user_account.rs
└── instructions/
    ├── initialize.rs
    ├── add_user.rs
    ├── remove_user.rs
    ├── init_extra_acc_meta.rs
    ├── transfer_hook.rs
    └── withdraw.rs
```

## Key PDAs

| Account | Seeds |
|---|---|
| Vault config | `["vault_config", admin_pubkey]` |
| User account | `["whitelist", user_pubkey]` |
| Extra account meta list | `["extra-account-metas", mint_pubkey]` |
