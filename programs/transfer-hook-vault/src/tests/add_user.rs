use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas, solana_program::{instruction::Instruction}};
use solana_keypair::Keypair;
use solana_signer::Signer;

use super::helper::*;

#[test]
fn test_add_user() {
    let (mut svm, admin) = setup();
    let (vault_pda, _mint_kp, _mint_pk) = do_initialize(&mut svm, &admin);

    let user = Keypair::new();
    let user_pk = user.pubkey();
    svm.airdrop(&user.pubkey(), 5 * 1_000_000_000).unwrap();

    let user_account_pda = do_add_user(&mut svm, &admin, &vault_pda, &user_pk);

    let user_acc = svm.get_account(&&user_account_pda).unwrap();
    let user_acc_data =
        crate::state::UserAccount::try_deserialize(&mut user_acc.data.as_ref()).unwrap();

    assert_eq!(user_acc_data.account, user_pk);
    assert_eq!(user_acc_data.amount, 0);
}

#[test]
fn test_remove_user() {
    let (mut svm, admin) = setup();
    let (vault_pda, _mint_kp, _mint_pk) = do_initialize(&mut svm, &admin);

    let user = Keypair::new();
    let user_pk = user.pubkey();

    let user_account_pda = do_add_user(&mut svm, &admin, &vault_pda, &user_pk);

    let admin_pk = admin.pubkey();
    let accounts = crate::accounts::RemoveUser {
        admin: admin_pk,
        vault: vault_pda,
        user_account: user_account_pda,
        system_program: system_program_id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts: convert_account_metas(accounts),
        data: crate::instruction::RemoveUser { address: user_pk }.data(),
    };

    send_ix(&mut svm, ix, &admin, &[]);

    let user_acc = svm.get_account(&&user_account_pda);
    assert!(user_acc.is_none(), "User account should be closed");
}
