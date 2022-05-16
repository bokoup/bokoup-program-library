use anchor_lang::prelude::*;
use mpl_token_metadata::state::{
    Collection as CollectionMpl, Creator as CreatorMpl, DataV2 as DataV2Mpl,
    UseMethod as UseMethodMpl, Uses as UsesMpl,
};

//==============================
// AdminSettings
//==============================

#[account]
#[derive(Default)]
pub struct AdminSettings {
    pub platform: Pubkey,
    pub create_promo_lamports: u64,
    pub redeem_promo_token_lamports: u64,
}

// Add extra space here when deployed to allow for additional settings
impl AdminSettings {
    pub const LEN: usize = 8
    + 32    // platform,
    + 32    // create_promo_lamports,
    + 32; // redeem_promo_token_lamports
}

//==============================
// Promo
//==============================

// mint.supply will equal number of issued tokens
// add max_issue_per account logic
// add option to restrict transfer - transfer author

#[account]
#[derive(PartialEq, Debug, Copy)]
pub struct Promo {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub metadata: Pubkey,
    pub mints: u32,
    pub burns: u32,
    pub max_mint: Option<u32>,
    pub max_burn: Option<u32>,
    pub expiry: Option<i64>,
}

impl Promo {
    pub const LEN: usize = 8
    + 32        // owner
    + 32        // mint
    + 32        // metadata
    + 16        // mints
    + 16        // burns
    + 1 + 4     // max_mint
    + 1 + 4     // max_redeem
    + 1 + 8; // expiry
}

//==============================
// Discount
//==============================

// add currency
//

#[account]
#[derive(PartialEq, Debug)]
pub struct Discount {
    pub buyer: Pubkey,
    pub promo: Pubkey,
    pub amount: u64,
    pub applied: bool,
}

impl Discount {
    pub const LEN: usize = 8
    + 32        // buyer
    + 32        // promo
    + 8         // amount
    + 1; // applied
}

//==============================
// Metadata
//==============================

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Debug, Clone)]
pub struct DataV2 {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
}

impl From<DataV2> for DataV2Mpl {
    fn from(item: DataV2) -> Self {
        DataV2Mpl {
            name: item.name,
            symbol: item.symbol,
            uri: item.uri,
            seller_fee_basis_points: item.seller_fee_basis_points,
            creators: item
                .creators
                .map(|a| a.into_iter().map(|v| v.into()).collect()),
            collection: item.collection.map(|v| v.into()),
            uses: item.uses.map(|v| v.into()),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Debug, Clone, Copy)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

impl From<Creator> for CreatorMpl {
    fn from(item: Creator) -> Self {
        CreatorMpl {
            address: item.address,
            verified: item.verified,
            share: item.share,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Debug, Clone, Copy)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Debug, Clone, Copy)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}

impl From<Uses> for UsesMpl {
    fn from(item: Uses) -> Self {
        UsesMpl {
            use_method: item.use_method.into(),
            remaining: item.remaining,
            total: item.total,
        }
    }
}

impl From<UseMethod> for UseMethodMpl {
    fn from(item: UseMethod) -> Self {
        match item {
            UseMethod::Burn => UseMethodMpl::Burn,
            UseMethod::Multiple => UseMethodMpl::Multiple,
            UseMethod::Single => UseMethodMpl::Single,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Debug, Clone, Copy)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

impl From<Collection> for CollectionMpl {
    fn from(item: Collection) -> Self {
        CollectionMpl {
            verified: item.verified,
            key: item.key,
        }
    }
}
