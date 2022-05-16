pub mod error;
pub mod processor;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;
use anchor_spl::{
    self,
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use borsh::BorshDeserialize;
use state::{AdminSettings, DataV2, Promo};
use utils::{ADMIN_PREFIX, AUTHORITY_PREFIX, PROMO_PREFIX};

declare_id!("CsmkSwyBPpihA6qiNGKtWR3DV6RNxJKBo4xBMPt414Eq");

#[program]
pub mod bpl_token_metadata {
    use super::*;

    pub fn create_admin_settings(
        ctx: Context<CreateAdminSettings>,
        data: AdminSettings,
    ) -> Result<()> {
        ctx.accounts.process(data)
    }

    pub fn create_promo(
        ctx: Context<CreatePromo>,
        promo_data: Promo,
        metadata_data: DataV2,
        is_mutable: bool,
    ) -> Result<()> {
        let authority_seeds = [AUTHORITY_PREFIX.as_bytes(), &[ctx.bumps[AUTHORITY_PREFIX]]];
        ctx.accounts
            .process(promo_data, metadata_data, is_mutable, authority_seeds)
    }

    pub fn mint_promo_token<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, MintPromoToken<'info>>,
    ) -> Result<()> {
        let authority_seeds = [AUTHORITY_PREFIX.as_bytes(), &[ctx.bumps[AUTHORITY_PREFIX]]];
        ctx.accounts.process(authority_seeds)
    }

    pub fn delegate_promo_token<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DelegatePromoToken<'info>>,
    ) -> Result<()> {
        ctx.accounts.process()
    }

    pub fn burn_promo_token<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, BurnPromoToken<'info>>,
    ) -> Result<()> {
        let authority_seeds = [AUTHORITY_PREFIX.as_bytes(), &[ctx.bumps[AUTHORITY_PREFIX]]];
        ctx.accounts.process(authority_seeds)
    }

    pub fn create_non_fungible(
        ctx: Context<CreateNonFungible>,
        data: DataV2,
        is_mutable: bool,
        max_supply: Option<u64>,
    ) -> Result<()> {
        let mint_authority_seeds = [AUTHORITY_PREFIX.as_bytes(), &[ctx.bumps[AUTHORITY_PREFIX]]];
        ctx.accounts
            .process(data, is_mutable, max_supply, mint_authority_seeds)
    }
}

// TODO: add program data check per anchor example
#[derive(Accounts)]
pub struct CreateAdminSettings<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, seeds = [ADMIN_PREFIX.as_bytes()], bump, payer = payer, space = AdminSettings::LEN)]
    pub admin_settings: Account<'info, AdminSettings>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts, Clone)]
#[instruction(promo_data: Promo, metadata_data: DataV2)]
pub struct CreatePromo<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(init, payer = payer, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub mint: Account<'info, Mint>,
    /// CHECK: Created via cpi
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(init, payer = payer, seeds = [PROMO_PREFIX.as_bytes(), mint.key().as_ref()], bump,
        constraint = promo_data.owner == payer.key(),
        space = Promo::LEN)]
    pub promo: Box<Account<'info, Promo>>,
    /// CHECK: pubkey checked via constraint
    #[account(mut, constraint = platform.key() == admin_settings.platform)]
    pub platform: UncheckedAccount<'info>,
    #[account(seeds = [ADMIN_PREFIX.as_bytes()], bump)]
    pub admin_settings: Box<Account<'info, AdminSettings>>,
    pub metadata_program: Program<'info, TokenMetadata>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts, Clone)]
pub struct TransferSol<'info> {
    /// CHECK: unchecked
    #[account(mut)]
    pub payer: AccountInfo<'info>,
    /// CHECK: unchecked
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts, Clone)]
pub struct MintPromoToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, constraint = promo_owner.key() == promo.owner)]
    pub promo_owner: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(mut, seeds = [PROMO_PREFIX.as_bytes(), mint.key().as_ref()], bump)]
    pub promo: Account<'info, Promo>,
    #[account(mut, seeds = [ADMIN_PREFIX.as_bytes()], bump)]
    pub admin_settings: Account<'info, AdminSettings>,
    #[account(init_if_needed, payer = payer, associated_token::mint = mint, associated_token::authority = payer)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts, Clone)]
pub struct DelegatePromoToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    pub promo: Account<'info, Promo>,
    #[account(mut, constraint = token_account.mint == promo.mint)]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts, Clone)]
pub struct BurnPromoToken<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, constraint = promo_owner.key() == promo.owner)]
    pub promo_owner: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(mut, seeds = [PROMO_PREFIX.as_bytes(), mint.key().as_ref()], bump)]
    pub promo: Account<'info, Promo>,
    #[account(mut, seeds = [ADMIN_PREFIX.as_bytes()], bump)]
    pub admin_settings: Account<'info, AdminSettings>,
    #[account(mut,
        constraint = token_account.mint == mint.key(),
        constraint = token_account.delegate.unwrap() == authority.key(),
        constraint = token_account.delegated_amount > 0,
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts, Clone)]
pub struct CreateNonFungible<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: pubkey checked via seeds
    #[account(seeds = [AUTHORITY_PREFIX.as_bytes()], bump)]
    pub authority: UncheckedAccount<'info>,
    #[account(init, payer = payer, mint::decimals = 0, mint::authority = authority, mint::freeze_authority = authority)]
    pub mint: Account<'info, Mint>,
    #[account(init, payer = payer, associated_token::mint = mint, associated_token::authority = authority)]
    pub token_account: Account<'info, TokenAccount>,
    /// CHECK: checked via cpi
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,
    /// CHECK: checked via cpi
    #[account(mut)]
    pub edition_account: UncheckedAccount<'info>,
    pub metadata_program: Program<'info, TokenMetadata>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts, Clone)]
pub struct CreateMetaData<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: checked via cpi
    pub metadata_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    /// CHECK: checked via cpi
    #[account(mut)]
    pub mint_authority: UncheckedAccount<'info>,
    /// CHECK: checked via cpi
    pub metadata_authority: UncheckedAccount<'info>,
    pub metadata_program: Program<'info, TokenMetadata>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> From<CreatePromo<'info>> for CreateMetaData<'info> {
    fn from(item: CreatePromo<'info>) -> Self {
        CreateMetaData {
            payer: item.payer,
            metadata_account: item.metadata,
            mint: item.mint,
            mint_authority: item.authority.clone(),
            metadata_authority: item.authority,
            metadata_program: item.metadata_program,
            rent: item.rent,
            system_program: item.system_program,
        }
    }
}

impl<'info> From<CreateNonFungible<'info>> for CreateMetaData<'info> {
    fn from(item: CreateNonFungible<'info>) -> Self {
        CreateMetaData {
            payer: item.payer,
            metadata_account: item.metadata_account,
            mint: item.mint,
            mint_authority: item.authority.clone(),
            metadata_authority: item.authority,
            metadata_program: item.metadata_program,
            rent: item.rent,
            system_program: item.system_program,
        }
    }
}

#[derive(Clone)]
pub struct TokenMetadata;

impl anchor_lang::Id for TokenMetadata {
    fn id() -> Pubkey {
        mpl_token_metadata::ID
    }
}

#[derive(AnchorDeserialize, Clone, Debug)]
pub struct Metadata(mpl_token_metadata::state::Metadata);

impl Metadata {
    pub const LEN: usize = mpl_token_metadata::state::MAX_METADATA_LEN;
}

impl anchor_lang::AccountDeserialize for Metadata {
    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self> {
        mpl_token_metadata::utils::try_from_slice_checked(
            buf,
            mpl_token_metadata::state::Key::MetadataV1,
            mpl_token_metadata::state::MAX_METADATA_LEN,
        )
        .map_err(Into::into)
    }
}

impl anchor_lang::AccountSerialize for Metadata {}

impl anchor_lang::Owner for Metadata {
    fn owner() -> Pubkey {
        mpl_token_metadata::ID
    }
}

impl core::ops::Deref for Metadata {
    type Target = mpl_token_metadata::state::Metadata;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
