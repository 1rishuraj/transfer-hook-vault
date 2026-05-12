use anchor_lang::prelude::Pubkey;
use anchor_lang::{InstructionData, ToAccountMetas, solana_program::{instruction::Instruction}};
use litesvm::LiteSVM;

use solana_keypair::Keypair;
use solana_message::{AccountMeta, Message};
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::constants::*;

pub static PROGRAM_ID: Pubkey = crate::ID;

pub fn token_2022_program_id() -> Pubkey {
    Pubkey::new_from_array(spl_token_2022::ID.to_bytes())
}

pub fn system_program_id() -> Pubkey {
    anchor_lang::system_program::ID
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * 1_000_000_000)
        .expect("Failed to airdrop SOL to payer");
    let program_id = crate::id();

     let bytes = include_bytes!("../../../../target/deploy/transfer_hook_vault.so");
    svm.add_program(program_id, bytes).unwrap();

    (svm, payer)
}

pub fn send_ix(svm: &mut LiteSVM, ix: Instruction, payer: &Keypair, extra_signers: &[&Keypair]) {
    send_ixs(svm, &[ix], payer, extra_signers);
}

pub fn send_ixs(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    extra_signers: &[&Keypair],
) {
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra_signers);

    let message = Message::new(ixs, Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&signers, message, recent_blockhash);
    svm.send_transaction(transaction).unwrap();
}

pub fn convert_account_metas(
    anchor_metas: Vec<anchor_lang::prelude::AccountMeta>,
) -> Vec<AccountMeta> {
    anchor_metas
        .into_iter()
        .map(|m| AccountMeta {
            pubkey: (m.pubkey),
            is_signer: m.is_signer,
            is_writable: m.is_writable,
        })
        .collect()
}

pub fn get_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address_with_program_id(
        owner,
        mint,
        &token_2022_program_id(),
    )
}

pub fn build_create_ata_ix(
    payer: Pubkey,
    ata: Pubkey,
    owner: Pubkey,
    mint: Pubkey,
) -> Instruction {
    Instruction {
        program_id: (anchor_spl::associated_token::ID),
        accounts: vec![
            AccountMeta {
                pubkey: payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: ata,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: owner,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: mint,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: (system_program_id()),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: (token_2022_program_id()),
                is_signer: false,
                is_writable: false,
            },
        ],
        data: vec![1],
    }
}

pub fn do_initialize(svm: &mut LiteSVM, admin: &Keypair) -> (Pubkey, Keypair, Pubkey) {
    let admin_pk = admin.pubkey();
    let mint = Keypair::new();
    let mint_pk = mint.pubkey();

    let vault_pda =
        Pubkey::find_program_address(&[VAULT_CONFIG.as_bytes(), admin_pk.as_ref()], &PROGRAM_ID).0;

    let accounts = crate::accounts::Initialize {
        admin: admin_pk,
        vault: vault_pda,
        mint: mint_pk,
        system_program: system_program_id(),
        token_program: token_2022_program_id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: (PROGRAM_ID),
        accounts: convert_account_metas(accounts),
        data: crate::instruction::Initialize {
            decimal: 9,
            name: "Vault Token".to_string(),
            symbol: "VAULT".to_string(),
            uri: "https://vault.example.com".to_string(),
        }
        .data(),
    };

    send_ix(svm, ix, admin, &[&mint]);
    (vault_pda, mint, mint_pk)
}

pub fn do_add_user(
    svm: &mut LiteSVM,
    admin: &Keypair,
    vault_pda: &Pubkey,
    user_pk: &Pubkey,
) -> Pubkey {
    let admin_pk = admin.pubkey();
    let user_account_pda =
        Pubkey::find_program_address(&[WHITELIST_ENTRY.as_bytes(), user_pk.as_ref()], &PROGRAM_ID)
            .0;

    let accounts = crate::accounts::AddUser {
        admin: admin_pk,
        vault: *vault_pda,
        user_account: user_account_pda,
        system_program: system_program_id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: (PROGRAM_ID),
        accounts: convert_account_metas(accounts),
        data: crate::instruction::AddUser { address: *user_pk }.data(),
    };

    send_ix(svm, ix, admin, &[]);
    user_account_pda
}

pub fn do_init_extra_acc_meta(
    svm: &mut LiteSVM,
    admin: &Keypair,
    mint_pk: &Pubkey,
    vault_pda: &Pubkey,
) -> Pubkey {
    let admin_pk = admin.pubkey();
    let extra_acc_meta_list = Pubkey::find_program_address(
        &[EXTRA_ACCOUNT_METAS.as_bytes(), mint_pk.as_ref()],
        &PROGRAM_ID,
    )
    .0;

    let accounts = crate::accounts::InitExtraAccountMeta {
        payer: admin_pk,
        extra_acc_meta_list,
        mint: *mint_pk,
        vault: *vault_pda,        // CHANGED: now wired in
        system_program: system_program_id(),
        token_program: token_2022_program_id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: convert_account_metas(accounts),
        data: crate::instruction::InitExtraAccMeta {}.data(),
    };

    send_ix(svm, ix, admin, &[]);
    extra_acc_meta_list
}

pub fn do_create_ata(svm: &mut LiteSVM, payer: &Keypair, owner: Pubkey, mint: Pubkey) -> Pubkey {
    let payer_pk = payer.pubkey();
    let ata = get_ata(&owner, &mint);
    let ix = build_create_ata_ix(payer_pk, ata, owner, mint);
    send_ix(svm, ix, payer, &[]);
    ata
}

pub fn do_mint_tokens(
    svm: &mut LiteSVM,
    admin: &Keypair,
    mint_pk: &Pubkey,
    to_ata: &Pubkey,
    amount: u64,
) {
    let admin_pk = &admin.pubkey();
    let mint_to_data = spl_token_2022::instruction::mint_to(
        &token_2022_program_id(),
        mint_pk,
        to_ata,
        &admin_pk,
        &[],
        amount,
    )
    .unwrap();

    let ix = Instruction {
        program_id: (token_2022_program_id()),
        accounts: mint_to_data
            .accounts
            .into_iter()
            .map(|m| AccountMeta {
                pubkey: (Pubkey::new_from_array(m.pubkey.to_bytes())),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: mint_to_data.data,
    };

    send_ix(svm, ix, admin, &[]);
}

// CHANGED: takes vault_pda; appended as writable extra account so hook can
// compare source/dest token owner against vault key, and whitelist is now
// writable since the hook mutates user_account.amount
#[allow(clippy::too_many_arguments)]
pub fn build_transfer_checked_ix(
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
    extra_account_meta_list: Pubkey,
    whitelist_pda: Pubkey,
    vault_pda: Pubkey, // CHANGED: added
) -> Instruction {
    let tc_base = spl_token_2022::instruction::transfer_checked(
        &token_2022_program_id(),
        source,
        mint,
        destination,
        authority,
        &[],
        amount,
        decimals,
    )
    .unwrap();

    let mut accounts: Vec<AccountMeta> = tc_base
        .accounts
        .into_iter()
        .map(|m| AccountMeta {
            pubkey: (Pubkey::new_from_array(m.pubkey.to_bytes())),
            is_signer: m.is_signer,
            is_writable: m.is_writable,
        })
        .collect();

    // extra_account_meta_list (index 4 in hook accounts)
    accounts.push(AccountMeta {
        pubkey: (extra_account_meta_list),
        is_signer: false,
        is_writable: false,
    });
    // whitelist PDA — CHANGED: writable, hook writes amount
    accounts.push(AccountMeta {
        pubkey: (whitelist_pda),
        is_signer: false,
        is_writable: true, // CHANGED
    });
    // vault PDA — CHANGED: added so hook can compare owner keys
    accounts.push(AccountMeta {
        pubkey: (vault_pda),
        is_signer: false,
        is_writable: false, // CHANGED: added
    });
    // hook program itself
    accounts.push(AccountMeta {
        pubkey: (PROGRAM_ID),
        is_signer: false,
        is_writable: false,
    });

    Instruction {
        program_id: (token_2022_program_id()),
        accounts,
        data: tc_base.data,
    }
}