# Transfer Hook Vault

A Solana program built with Anchor that uses the **Token-2022 Transfer Hook** extension to enforce a whitelist on every token transfer. Only admin-approved users can hold or transfer the vault's token.

## How It Works

- The admin initializes a vault and a Token-2022 mint with `TransferHook`, `MetadataPointer`, and `TokenMetadata` extensions.
- The admin whitelists users via `add_user` / `remove_user`.
- On every `transfer_checked`, Token-2022 automatically calls the `transfer_hook` instruction, which checks that the sender is whitelisted and updates their on-chain balance ledger.
- **Deposit**: send `[deposit ix, transfer_checked ix]` atomically — the hook records the deposit.
- **Withdraw**: send `[withdraw ix (approves delegate), transfer_checked ix]` atomically — the hook records the withdrawal.

> Deposit and withdraw are split from the actual token transfer to avoid Token-2022's reentrancy restriction.

## Instructions

| Instruction | Who | Description |
|---|---|---|
| `initialize` | Admin | Creates vault PDA + Token-2022 mint |
| `add_user` | Admin | Whitelists a user |
| `remove_user` | Admin | Removes a user from whitelist |
| `init_extra_acc_meta` | Anyone | Registers extra accounts for the hook |
| `transfer_hook` | Token-2022 (auto) | Validates sender is whitelisted |
| `withdraw` | User | Approves delegate + decrements ledger |

## Build & Test

```bash
make build   # build the program
make test    # run all tests (uses LiteSVM — no local validator needed)
make all     # clean, build, and test
```

## Tech Stack

- **Anchor** — Solana framework
- **Token-2022** — SPL token with Transfer Hook extension
- **LiteSVM** — in-process Solana VM for fast testing
