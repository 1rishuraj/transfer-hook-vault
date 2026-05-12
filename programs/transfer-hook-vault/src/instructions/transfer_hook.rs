use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_token_2022::extension::BaseStateWithExtensions;
use spl_token_2022::extension::{transfer_hook::TransferHookAccount, PodStateWithExtensionsMut};
use spl_token_2022::pod::PodAccount;

use crate::{UserAccount, Vault, EXTRA_ACCOUNT_METAS, VAULT_CONFIG, WHITELIST_ENTRY};

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::token_program = anchor_spl::token_2022::ID,
    )]
    pub source_token: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        token::mint = mint,
        token::token_program = anchor_spl::token_2022::ID,
    )]
    pub destination_token: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    /// CHECK: Owner of source token, passed by Token-2022
    pub owner: UncheckedAccount<'info>,

    /// CHECK: Extra account meta list PDA
    #[account(
        seeds = [EXTRA_ACCOUNT_METAS.as_bytes(), mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,


    #[account(
        mut,                         
        seeds = [WHITELIST_ENTRY.as_bytes(), owner.key().as_ref()],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, UserAccount>,

    #[account(
        seeds = [VAULT_CONFIG.as_bytes(), vault.admin.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(&mut self, amount: u64) -> Result<()> {
        self.check_is_transferring()?;

        let vault_key = self.vault.key();

        // CASE 1: Deposit — tokens moving INTO the vault ATA
        // source owner is the whitelisted user, destination owner is the vault PDA
        if self.destination_token.owner == vault_key {
            // Whitelist check still enforced: only whitelisted users can deposit
            require_keys_eq!(
                self.whitelist.account,
                self.owner.key(),
                anchor_lang::error::ErrorCode::ConstraintOwner
            );

            self.whitelist.amount = self
                .whitelist
                .amount
                .checked_add(amount)
                .ok_or(crate::error::ErrorCode::Overflow)?;

            msg!(
                "Hook: recorded deposit of {} tokens for {}",
                amount,
                self.owner.key()
            );
        }
        // source owner is the vault PDA, destination owner is the user
        // CASE 2: Withdrawal — tokens moving OUT of the vault ATA
        else if self.source_token.owner == vault_key {
            require_keys_eq!(
                self.whitelist.account,
                self.owner.key(),
                anchor_lang::error::ErrorCode::ConstraintOwner
            );

            self.whitelist.amount = self
                .whitelist
                .amount
                .checked_sub(amount)
                .ok_or(crate::error::ErrorCode::InsufficientFunds)?;

            msg!(
                "Hook: recorded withdrawal of {} tokens for {}",
                amount,
                self.owner.key()
            );
        }
        // CASE 3: Regular whitelisted transfer (user-to-user)
        else {
            require_keys_eq!(
                self.whitelist.account,
                self.owner.key(),
                anchor_lang::error::ErrorCode::ConstraintOwner
            );
            msg!("Transfer allowed: {} is whitelisted", self.owner.key());
        }

        Ok(())
    }

    fn check_is_transferring(&self) -> Result<()> {
        let source_token_info = self.source_token.to_account_info();
        let mut account_data = source_token_info.try_borrow_mut_data()?;
        let account = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account_data)?;
        let transfer_hook = account.get_extension::<TransferHookAccount>()?;

        if !bool::from(transfer_hook.transferring) {
            return err!(anchor_lang::error::ErrorCode::AccountNotInitialized);
        }

        Ok(())
    }
}
