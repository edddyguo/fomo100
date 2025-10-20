// 生成一份在solana生态创建一个nft collection的代码，在合约中生成，这个collection的权限也属于该合约
// 将下边的代码进行调整，创建好的collection权限为当前合约的根据seed的派生地址，
// 并且mint的权限也属于该合约

use std::convert::TryInto;

use crate::errors::MinterError;
use crate::state::COLLECTION_AUTHORITY_SEED;
use crate::state::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
    CreateMetadataAccountsV3, Metadata,
};
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use mpl_token_metadata::types::{Collection, Creator, DataV2};
use crate::constants;

fn string_to_array<const N: usize>(s: String) -> [u8; N] {
    let mut array = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N);
    array[..len].copy_from_slice(&bytes[..len]);
    array
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds=[ADMIN_STATE_SEED.as_bytes()], 
        bump,
        constraint = admin_state.admin.key() == admin.key() @ MinterError::InsufficientPermission
    )]
    admin_state: Account<'info, AdminState>,
    #[account(
        init,
        payer = admin,
        space = 8 + CollectionState::LEN,
        seeds = [COLLECTION_STATE_SEED.as_bytes(),collection_mint.key().as_ref()],
        bump,
    )]
    pub collection_state: Account<'info, CollectionState>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 0,
        mint::authority = collection_authority,
        mint::freeze_authority = collection_authority,
        seeds = [COLLECTION_MINT_SEED.as_bytes(),name.as_bytes()],
        bump
    )]
    pub collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = collection_mint,
        associated_token::authority = collection_authority
    )]
    pub collection_token_account: Account<'info, TokenAccount>,

    /// CHECK: Metadata account
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Master edition account
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,
    /// CHECK: Collection authority account
    #[account(
        mut,
        seeds = [COLLECTION_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    pub collection_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateCollection>,
    name: String,
    symbol: String,
    base_uri: String,
    sol_price: u64,
    settle_token_price: u64,
    settle_token: Pubkey,
) -> Result<()> {
    require_eq!(
        base_uri.contains("collection"),
        true,
        MinterError::InvalidUri
    );

    let uri = base_uri.clone();
    // 先初始化 collection state，避免后续的栈帧访问问题
    let collection_state = &mut ctx.accounts.collection_state;
    collection_state.name = string_to_array(name.clone());
    collection_state.symbol = string_to_array(symbol.clone());
    collection_state.base_uri = string_to_array(base_uri);
    collection_state.amount = 0;
    collection_state.sol_price = sol_price;
    collection_state.sol_income = 0;
    collection_state.settle_token = settle_token;
    collection_state.settle_token_price = settle_token_price;
    collection_state.settle_token_income = 0;

    let authority_seeds = &[
        COLLECTION_AUTHORITY_SEED.as_bytes(),
        &[ctx.bumps.collection_authority],
    ];

    // Mint 1 token for collection
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.collection_mint.to_account_info(),
                to: ctx.accounts.collection_token_account.to_account_info(),
                authority: ctx.accounts.collection_authority.to_account_info(),
            },
            &[authority_seeds],
        ),
        1,
    )?;

    // Create metadata account
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_authority.to_account_info(),
                payer: ctx.accounts.admin.to_account_info(),
                update_authority: ctx.accounts.collection_authority.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &[authority_seeds],
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        true,
        true,
        None,
    )?;

    // Create master edition
    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.collection_master_edition.to_account_info(),
                mint: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_authority.to_account_info(),
                mint_authority: ctx.accounts.collection_authority.to_account_info(),
                metadata: ctx.accounts.collection_metadata.to_account_info(),
                payer: ctx.accounts.admin.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &[authority_seeds],
        ),
        Some(0), // Max supply of 0 means unlimited
    )?;

    msg!("Collection created successfully");
    Ok(())
}
