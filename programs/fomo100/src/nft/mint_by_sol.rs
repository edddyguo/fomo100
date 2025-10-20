use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
    CreateMetadataAccountsV3, Metadata,
};
use anchor_spl::metadata::{set_and_verify_collection, verify_collection};
use anchor_spl::metadata::{SetAndVerifyCollection, VerifyCollection};
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use mpl_token_metadata::types::{Collection, Creator, DataV2};
use crate::constants;

use crate::errors::MinterError;
use crate::state::*;
use anchor_spl::token::{self, Transfer};
use anchor_spl::token_interface::TokenAccount as TokenAccountInterface;

fn array_to_string(array: &[u8]) -> String {
    println!("array: {:?}", array);
    let len = array.iter().position(|&x| x == 0).unwrap_or(8); // 找到第一个 0 或默认 32
    String::from_utf8_lossy(&array[..len]).into_owned()
}

pub fn handler(ctx: Context<NftMintBySol>, id: u32) -> Result<()> {
    let nft_uri = array_to_string(&ctx.accounts.collection_state.base_uri).replace("collection", "nft");
    let uri = format!("{}/{}.json", nft_uri,id);
    let collection_mint_key = ctx.accounts.collection_mint.key();
    let seeds = &[
        MINT_SEED.as_bytes(),
        &collection_mint_key.as_ref(),
        &id.to_le_bytes()[..],
        &[ctx.bumps.mint],
    ];

    let seeds_collection = &[
        COLLECTION_AUTHORITY_SEED.as_bytes(),
        &[ctx.bumps.collection_authority],
    ];

    msg!("Run mint_to");

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                authority: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
            &[&seeds[..]],
        ),
        1, // 1 token
    )?;

    msg!("Run create metadata accounts v3");

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &[&seeds[..], &seeds_collection[..]],
        ),
        DataV2 {
            name: format!(
                "{} #{}",
                array_to_string(&ctx.accounts.collection_state.name),
                id
            ),
            symbol: array_to_string(&ctx.accounts.collection_state.symbol),
            uri, //array_to_string(&ctx.accounts.collection_state.base_uri),
            seller_fee_basis_points: 0,
            creators: None,
            collection: Some(mpl_token_metadata::types::Collection {
                key: ctx.accounts.collection_mint.key(),
                verified: false,
            }),
            uses: None,
        },
        false,
        true,
        None,
    )?;

    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.nft_master_edition.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.payer.to_account_info(),
                mint_authority: ctx.accounts.payer.to_account_info(),
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &[seeds],
        ),
        Some(0), // Max supply of 0 means unlimited
    )?;

    msg!("Minted NFT successfully");

    //generate code: call set_and_verify_collection func to verify collection
    let collection_master_edition = ctx.accounts.collection_master_edition.key();
    let collection_authority = ctx.accounts.collection_authority.key();

    let seeds_collection = &[
        COLLECTION_AUTHORITY_SEED.as_bytes(),
        &[ctx.bumps.collection_authority],
    ];

    verify_collection(
        CpiContext::new_with_signer(
            ctx.accounts.metadata_program.to_account_info(),
            VerifyCollection {
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                collection_authority: ctx.accounts.collection_authority.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                collection_mint: ctx.accounts.collection_mint.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
            },
            &[&[
                COLLECTION_AUTHORITY_SEED.as_bytes(),
                &[ctx.bumps.collection_authority],
            ]],
        ),
        None,
    )?;


    ctx.accounts.collection_state.amount += 1;
    if ctx.accounts.user_state.pending_airdrop_count > 0 {
        ctx.accounts.user_state.claimed_airdrop_count += 1;
        ctx.accounts.user_state.pending_airdrop_count -= 1;
    } else {
        msg!("pending_airdrop_count is zero, start pay sol");
        let pay_amount = ctx.accounts.collection_state.sol_price;
        ctx.accounts.collection_state.sol_income += pay_amount;
        // 执行 SOL 转账
        let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.payer.key(),
            &ctx.accounts.admin_state.treasurer,
            pay_amount,
        );
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.treasurer.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        ctx.accounts.user_state.payed_count += 1;
    }
    Ok(())
}

//区别于原来的fomo100,如今ata在派生宏里面创建，
#[derive(Accounts)]
#[instruction(id: u32)]
pub struct NftMintBySol<'info> {
    #[account(mut)]
    /// CHECK: This account is the authority and must be mutable
    pub payer: Signer<'info>, // Ensure this is the correct authority
    #[account(
        mut,
        seeds=[USER_STATE_SEED.as_bytes(),collection_mint.key().as_ref(),payer.key().as_ref()], 
        bump,
    )]
    user_state: Box<Account<'info, UserState>>,
    #[account(
        mut,
        seeds = [COLLECTION_STATE_SEED.as_bytes(),collection_mint.key().as_ref()],
        bump,
    )]
    pub collection_state: Box<Account<'info, CollectionState>>,

    #[account(
        init,
        payer = payer, 
        mint::decimals = 0,
        mint::authority = payer.key(), // Ensure this matches the authority
        mint::freeze_authority = payer.key(), // Ensure this matches the authority
        seeds = [MINT_SEED.as_bytes(),collection_mint.key().as_ref(),&id.to_le_bytes()[..]], 
        bump,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    pub token_account: Box<Account<'info, TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub metadata_program: Program<'info, Metadata>,
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = metadata_program.key()
    )]
    /// CHECK: This account must be mutable
    pub nft_metadata: UncheckedAccount<'info>,
    /// CHECK: This account must be mutable
    #[account(mut)]
    pub nft_master_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [COLLECTION_AUTHORITY_SEED.as_bytes()],
        bump
    )]
    /// CHECK: Collection authority derived from seed
    pub collection_authority: AccountInfo<'info>,

    pub collection_mint: Box<Account<'info, Mint>>,
    /// CHECK: Collection metadata account
    #[account(mut)]
    pub collection_metadata: UncheckedAccount<'info>,

    /// CHECK: Master edition account
    #[account(mut)]
    pub collection_master_edition: UncheckedAccount<'info>,

    #[account(        
        seeds=[ADMIN_STATE_SEED.as_bytes()], 
        bump,
    )]
    admin_state: Box<Account<'info, AdminState>>,

    /// CHECK: This account must be mutable
    #[account(mut, constraint = treasurer.key() == admin_state.treasurer @ MinterError::NotTreasurer)]
    treasurer: AccountInfo<'info>,
}
