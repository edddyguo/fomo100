use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
    CreateMetadataAccountsV3, Metadata,
};
use crate::constants::INIT_AIRDROP_SIGN_PREFIX;
use anchor_spl::metadata::{set_and_verify_collection, verify_collection};
use anchor_spl::metadata::{SetAndVerifyCollection, VerifyCollection};
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use mpl_token_metadata::types::{Collection, Creator, DataV2};
use spl_token::solana_program::ed25519_program;
use std::convert::TryInto;
//use solana_program::sysvar::instructions::{ID as IX_ID};
use spl_token::solana_program::sysvar::instructions::{load_instruction_at_checked, ID as IX_ID};

// 初始化 user_state
// 在有额度的时候，基于无额度进行初始化

fn verify(ctx: &Context<InitAirdrop>, i: usize, validator_pubkey: &Pubkey, msg: String, sig: &[u8; 64]) -> Result<()> {
    let ix = load_instruction_at_checked(i, &ctx.accounts.ix_sysvar)?;
    // Check that ix is what we expect to have been sent
    crate::utils::verify_ed25519_ix(&ix, validator_pubkey.as_ref(), &msg.as_bytes(), sig)?;
    Ok(())
}
pub fn handler(ctx: Context<InitAirdrop>, amount: u32, sig: [u8; 64]) -> Result<()> {
    
        let public_key = ctx.accounts.admin_state.validator;
        let msg = format!(
            "{}_{}_{}_{}",
            INIT_AIRDROP_SIGN_PREFIX,
            ctx.accounts.collection_mint.key().to_string(),
            ctx.accounts.payer.key().to_string(),
            amount
        );
        msg!("start verify msg {}", msg);
        
        //考虑到有些客户端增加手续费的指令，此处需要多检查几次
        for i in 0..10usize {
            if verify(&ctx, i,&public_key, msg.clone(), &sig).is_ok() {
                break;
            }
        }

        msg!("Signature verified successfully for message: {:?}", msg);
        ctx.accounts.user_state.pending_airdrop_count = amount;
    Ok(())
}

//区别于原来的fomo100,如今ata在派生宏里面创建，
#[derive(Accounts)]
//#[instruction(collection_name: String)]
pub struct InitAirdrop<'info> {
    #[account(mut)]
    /// CHECK: This account is the authority and must be mutable
    pub payer: Signer<'info>, // Ensure this is the correct authority
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + UserState::LEN,
        seeds=[USER_STATE_SEED.as_bytes(),collection_mint.key().as_ref(),payer.key().as_ref()], 
        bump,
    )]
    user_state: Account<'info, UserState>,
    ///
    #[account(        
        seeds=[ADMIN_STATE_SEED.as_bytes()], 
        bump,
    )]
    admin_state: Box<Account<'info, AdminState>>,
    ///
    pub collection_mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This account must be mutable
    #[account(address = IX_ID)]
    pub ix_sysvar: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
