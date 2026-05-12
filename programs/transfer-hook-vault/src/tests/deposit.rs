use anchor_lang::AccountDeserialize;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_token_2022::extension::StateWithExtensions;
use spl_token_2022::state::Account as TokenAccount;

use super::helper::*;

#[test]
fn test_deposit() {
    let (mut svm, admin) = setup();
    let (vault_pda, _mint_kp, mint_pk) = do_initialize(&mut svm, &admin);

    let user = Keypair::new();
    let user_pk = user.pubkey();
    svm.airdrop(&user.pubkey(), 5 * 1_000_000_000).unwrap();

    let user_account_pda = do_add_user(&mut svm, &admin, &vault_pda, &user_pk);
    let extra_acc_meta_list = do_init_extra_acc_meta(&mut svm, &admin, &mint_pk, &vault_pda);

    let user_ata = do_create_ata(&mut svm, &user, user_pk, mint_pk);
    let vault_ata = do_create_ata(&mut svm, &admin, vault_pda, mint_pk);

    let mint_amount: u64 = 1_000_000_000_000;
    do_mint_tokens(&mut svm, &admin, &mint_pk, &user_ata, mint_amount);

    let deposit_amount: u64 = 500_000_000_000;

    let transfer_ix = build_transfer_checked_ix(
        &user_ata,
        &mint_pk,
        &vault_ata,
        &user_pk,
        deposit_amount,
        9,
        extra_acc_meta_list,
        user_account_pda,
        vault_pda, 
    );

    send_ixs(&mut svm, &[transfer_ix], &user, &[]);

    let vault_ata_account = svm.get_account(&&vault_ata).unwrap();
    let vault_token_data =
        StateWithExtensions::<TokenAccount>::unpack(&vault_ata_account.data).unwrap();
    assert_eq!(vault_token_data.base.amount, deposit_amount);

    let user_ata_account = svm.get_account(&&user_ata).unwrap();
    let user_token_data =
        StateWithExtensions::<TokenAccount>::unpack(&user_ata_account.data).unwrap();
    assert_eq!(user_token_data.base.amount, mint_amount - deposit_amount);

    let user_acc = svm.get_account(&&user_account_pda).unwrap();
    let user_acc_data =
        crate::state::UserAccount::try_deserialize(&mut user_acc.data.as_ref()).unwrap();
    assert_eq!(user_acc_data.amount, deposit_amount);
}