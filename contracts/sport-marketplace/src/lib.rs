#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, String, Symbol, Vec,
    token::Client as TokenClient,
    log,
};

// ─────────────────────────────────────────────
//  Storage Keys
// ─────────────────────────────────────────────
const ADMIN_KEY: Symbol         = symbol_short!("ADMIN");
const LISTING_COUNT_KEY: Symbol = symbol_short!("LIST_CNT");
const FEE_BPS_KEY: Symbol       = symbol_short!("FEE_BPS");   // basis points (100 = 1 %)

// ─────────────────────────────────────────────
//  Data Types
// ─────────────────────────────────────────────

/// Category of the sport item being listed.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum SportCategory {
    Football,
    Basketball,
    Tennis,
    Baseball,
    Golf,
    Swimming,
    Cycling,
    Running,
    Other,
}

/// Condition of the item.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ItemCondition {
    New,
    LikeNew,
    Good,
    Fair,
    Poor,
}

/// Status of a marketplace listing.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ListingStatus {
    Active,
    Unlisted,
    Sold,
}

/// A single marketplace listing.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Listing {
    pub id:          u64,
    pub seller:      Address,
    pub title:       String,
    pub description: String,
    pub category:    SportCategory,
    pub condition:   ItemCondition,
    pub price:       i128,          // price in token stroops
    pub token:       Address,       // payment token (XLM or any SAC token)
    pub status:      ListingStatus,
    pub created_at:  u64,           // ledger timestamp
    pub updated_at:  u64,
}

/// Lightweight summary stored in the global index.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ListingIndex {
    pub id:       u64,
    pub seller:   Address,
    pub price:    i128,
    pub status:   ListingStatus,
    pub category: SportCategory,
}

// ─────────────────────────────────────────────
//  Error Codes
// ─────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum MarketError {
    NotFound         = 1,
    Unauthorized     = 2,
    NotActive        = 3,
    AlreadyUnlisted  = 4,
    SelfPurchase     = 5,
    InvalidPrice     = 6,
    InvalidFeeBps    = 7,
    InsufficientFunds = 8,
}

// ─────────────────────────────────────────────
//  Storage helpers
// ─────────────────────────────────────────────
fn listing_key(id: u64) -> (Symbol, u64) {
    (symbol_short!("LISTING"), id)
}

fn index_key() -> Symbol {
    symbol_short!("INDEX")
}

fn fee_vault_key() -> Symbol {
    symbol_short!("VAULT")
}

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────
#[contract]
pub struct SportMarketplace;

#[contractimpl]
impl SportMarketplace {

    // ──────────────────────────────────────────
    //  Initialization
    // ──────────────────────────────────────────

    /// Initialize the marketplace.
    /// `fee_bps`  – platform fee in basis points (e.g. 250 = 2.5 %)
    /// `fee_vault`– address that collects platform fees
    pub fn initialize(
        env:       Env,
        admin:     Address,
        fee_bps:   u32,
        fee_vault: Address,
    ) {
        assert!(
            !env.storage().instance().has(&ADMIN_KEY),
            "Already initialized"
        );
        assert!(fee_bps <= 10_000, "fee_bps must be <= 10000");

        admin.require_auth();

        env.storage().instance().set(&ADMIN_KEY,         &admin);
        env.storage().instance().set(&FEE_BPS_KEY,       &fee_bps);
        env.storage().instance().set(&LISTING_COUNT_KEY, &0u64);
        env.storage().instance().set(&fee_vault_key(),   &fee_vault);

        // Empty index
        let index: Vec<ListingIndex> = Vec::new(&env);
        env.storage().instance().set(&index_key(), &index);

        log!(&env, "SportMarketplace initialized by {}", admin);
    }

    // ──────────────────────────────────────────
    //  Admin
    // ──────────────────────────────────────────

    /// Update the platform fee.  Only admin.
    pub fn set_fee_bps(env: Env, fee_bps: u32) {
        Self::require_admin(&env);
        assert!(fee_bps <= 10_000, "fee_bps must be <= 10000");
        env.storage().instance().set(&FEE_BPS_KEY, &fee_bps);
    }

    /// Update fee vault address. Only admin.
    pub fn set_fee_vault(env: Env, new_vault: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&fee_vault_key(), &new_vault);
    }

    // ──────────────────────────────────────────
    //  List  (create a new listing)
    // ──────────────────────────────────────────

    /// List a sport item for sale.
    /// Returns the new listing id.
    pub fn list_item(
        env:         Env,
        seller:      Address,
        title:       String,
        description: String,
        category:    SportCategory,
        condition:   ItemCondition,
        price:       i128,
        token:       Address,
    ) -> u64 {
        seller.require_auth();
        assert!(price > 0, "Price must be positive");

        let id: u64 = env.storage().instance().get(&LISTING_COUNT_KEY).unwrap_or(0);
        let now      = env.ledger().timestamp();

        let listing = Listing {
            id,
            seller:      seller.clone(),
            title:       title.clone(),
            description,
            category:    category.clone(),
            condition,
            price,
            token,
            status:      ListingStatus::Active,
            created_at:  now,
            updated_at:  now,
        };

        // Persist full listing
        env.storage().persistent().set(&listing_key(id), &listing);

        // Update global index
        let mut index: Vec<ListingIndex> =
            env.storage().instance().get(&index_key()).unwrap_or_else(|| Vec::new(&env));

        index.push_back(ListingIndex {
            id,
            seller,
            price,
            status:   ListingStatus::Active,
            category,
        });
        env.storage().instance().set(&index_key(), &index);

        // Bump listing count
        env.storage().instance().set(&LISTING_COUNT_KEY, &(id + 1));

        log!(&env, "Item listed: id={} title={} price={}", id, title, price);
        id
    }

    // ──────────────────────────────────────────
    //  Unlist  (delist by the seller)
    // ──────────────────────────────────────────

    /// Remove a listing from the marketplace. Only the seller can do this.
    pub fn unlist_item(env: Env, seller: Address, listing_id: u64) {
        seller.require_auth();

        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&listing_key(listing_id))
            .expect("Listing not found");

        assert!(listing.seller == seller,           "Not the seller");
        assert!(listing.status == ListingStatus::Active, "Listing is not active");

        listing.status     = ListingStatus::Unlisted;
        listing.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(&listing_key(listing_id), &listing);
        Self::update_index_status(&env, listing_id, ListingStatus::Unlisted);

        log!(&env, "Item unlisted: id={}", listing_id);
    }

    // ──────────────────────────────────────────
    //  Re-list  (seller puts it back for sale)
    // ──────────────────────────────────────────

    /// Re-activate an unlisted item, optionally with a new price.
    pub fn relist_item(
        env:        Env,
        seller:     Address,
        listing_id: u64,
        new_price:  Option<i128>,
    ) {
        seller.require_auth();

        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&listing_key(listing_id))
            .expect("Listing not found");

        assert!(listing.seller == seller,               "Not the seller");
        assert!(listing.status == ListingStatus::Unlisted, "Can only relist an unlisted item");

        if let Some(p) = new_price {
            assert!(p > 0, "Price must be positive");
            listing.price = p;
        }

        listing.status     = ListingStatus::Active;
        listing.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(&listing_key(listing_id), &listing);
        Self::update_index_status(&env, listing_id, ListingStatus::Active);

        log!(&env, "Item relisted: id={}", listing_id);
    }

    // ──────────────────────────────────────────
    //  Update price
    // ──────────────────────────────────────────

    /// Let the seller change the price of an active listing.
    pub fn update_price(
        env:        Env,
        seller:     Address,
        listing_id: u64,
        new_price:  i128,
    ) {
        seller.require_auth();
        assert!(new_price > 0, "Price must be positive");

        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&listing_key(listing_id))
            .expect("Listing not found");

        assert!(listing.seller == seller,           "Not the seller");
        assert!(listing.status == ListingStatus::Active, "Listing is not active");

        listing.price      = new_price;
        listing.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(&listing_key(listing_id), &listing);

        // Also update the index entry price
        let mut index: Vec<ListingIndex> =
            env.storage().instance().get(&index_key()).unwrap_or_else(|| Vec::new(&env));

        for i in 0..index.len() {
            let mut entry = index.get(i).unwrap();
            if entry.id == listing_id {
                entry.price = new_price;
                index.set(i, entry);
                break;
            }
        }
        env.storage().instance().set(&index_key(), &index);

        log!(&env, "Price updated: id={} new_price={}", listing_id, new_price);
    }

    // ──────────────────────────────────────────
    //  Buy
    // ──────────────────────────────────────────

    /// Purchase a listed item.
    /// The buyer must have approved this contract to spend `listing.price` tokens.
    pub fn buy_item(env: Env, buyer: Address, listing_id: u64) {
        buyer.require_auth();

        let mut listing: Listing = env
            .storage()
            .persistent()
            .get(&listing_key(listing_id))
            .expect("Listing not found");

        assert!(listing.status == ListingStatus::Active, "Listing is not active");
        assert!(listing.seller != buyer,               "Cannot buy your own listing");

        let fee_bps:  u32     = env.storage().instance().get(&FEE_BPS_KEY).unwrap_or(0);
        let fee_vault: Address = env.storage().instance().get(&fee_vault_key()).unwrap();

        let fee_amount: i128 = (listing.price * fee_bps as i128) / 10_000;
        let seller_amount    = listing.price - fee_amount;

        let token = TokenClient::new(&env, &listing.token);

        // Transfer full price from buyer → contract  (contract is the escrow step)
        // In Soroban the simplest pattern is direct transfers:
        //   buyer → seller   (seller_amount)
        //   buyer → vault    (fee_amount)
        token.transfer(&buyer, &listing.seller, &seller_amount);

        if fee_amount > 0 {
            token.transfer(&buyer, &fee_vault, &fee_amount);
        }

        // Mark as sold
        listing.status     = ListingStatus::Sold;
        listing.updated_at = env.ledger().timestamp();

        env.storage().persistent().set(&listing_key(listing_id), &listing);
        Self::update_index_status(&env, listing_id, ListingStatus::Sold);

        log!(
            &env,
            "Item sold: id={} buyer={} seller={} price={} fee={}",
            listing_id, buyer, listing.seller, listing.price, fee_amount
        );
    }

    // ──────────────────────────────────────────
    //  Read-only queries
    // ──────────────────────────────────────────

    /// Get full details of a listing by id.
    pub fn get_listing(env: Env, listing_id: u64) -> Option<Listing> {
        env.storage().persistent().get(&listing_key(listing_id))
    }

    /// Return the lightweight index (all listings, any status).
    pub fn get_all_listings(env: Env) -> Vec<ListingIndex> {
        env.storage()
            .instance()
            .get(&index_key())
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Return only active listings.
    pub fn get_active_listings(env: Env) -> Vec<ListingIndex> {
        let all: Vec<ListingIndex> = env
            .storage()
            .instance()
            .get(&index_key())
            .unwrap_or_else(|| Vec::new(&env));

        let mut active = Vec::new(&env);
        for i in 0..all.len() {
            let entry = all.get(i).unwrap();
            if entry.status == ListingStatus::Active {
                active.push_back(entry);
            }
        }
        active
    }

    /// Return listings for a given seller address.
    pub fn get_seller_listings(env: Env, seller: Address) -> Vec<ListingIndex> {
        let all: Vec<ListingIndex> = env
            .storage()
            .instance()
            .get(&index_key())
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        for i in 0..all.len() {
            let entry = all.get(i).unwrap();
            if entry.seller == seller {
                result.push_back(entry);
            }
        }
        result
    }

    /// Return active listings filtered by category.
    pub fn get_listings_by_category(env: Env, category: SportCategory) -> Vec<ListingIndex> {
        let all: Vec<ListingIndex> = env
            .storage()
            .instance()
            .get(&index_key())
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        for i in 0..all.len() {
            let entry = all.get(i).unwrap();
            if entry.category == category && entry.status == ListingStatus::Active {
                result.push_back(entry);
            }
        }
        result
    }

    /// Get the total number of listings ever created.
    pub fn get_listing_count(env: Env) -> u64 {
        env.storage().instance().get(&LISTING_COUNT_KEY).unwrap_or(0)
    }

    /// Get current platform fee in basis points.
    pub fn get_fee_bps(env: Env) -> u32 {
        env.storage().instance().get(&FEE_BPS_KEY).unwrap_or(0)
    }

    /// Get the admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN_KEY).unwrap()
    }

    // ──────────────────────────────────────────
    //  Internal helpers
    // ──────────────────────────────────────────

    fn require_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
    }

    fn update_index_status(env: &Env, listing_id: u64, new_status: ListingStatus) {
        let mut index: Vec<ListingIndex> = env
            .storage()
            .instance()
            .get(&index_key())
            .unwrap_or_else(|| Vec::new(env));

        for i in 0..index.len() {
            let mut entry = index.get(i).unwrap();
            if entry.id == listing_id {
                entry.status = new_status;
                index.set(i, entry);
                break;
            }
        }
        env.storage().instance().set(&index_key(), &index);
    }
}

// ─────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        Env, String,
    };

    fn setup_env() -> (Env, Address, Address, Address, Address) {
        let env     = Env::default();
        env.mock_all_auths();

        let admin  = Address::generate(&env);
        let vault  = Address::generate(&env);
        let seller = Address::generate(&env);
        let buyer  = Address::generate(&env);

        (env, admin, vault, seller, buyer)
    }

    fn create_token(env: &Env, admin: &Address) -> Address {
        // Use the built-in stellar_asset_contract for testing
        let token_contract_id = env.register_stellar_asset_contract_v2(admin.clone());
        token_contract_id.address()
    }

    #[test]
    fn test_initialize() {
        let (env, admin, vault, _seller, _buyer) = setup_env();
        let contract_id = env.register(SportMarketplace, ());
        let client      = SportMarketplaceClient::new(&env, &contract_id);

        client.initialize(&admin, &250, &vault);

        assert_eq!(client.get_fee_bps(), 250);
        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_listing_count(), 0);
    }

    #[test]
    fn test_list_and_get() {
        let (env, admin, vault, seller, _buyer) = setup_env();
        let contract_id = env.register(SportMarketplace, ());
        let client      = SportMarketplaceClient::new(&env, &contract_id);
        let token       = create_token(&env, &admin);

        client.initialize(&admin, &250, &vault);

        let id = client.list_item(
            &seller,
            &String::from_str(&env, "Nike Football Boots"),
            &String::from_str(&env, "Size 10, barely used"),
            &SportCategory::Football,
            &ItemCondition::LikeNew,
            &50_000_000,
            &token,
        );

        assert_eq!(id, 0);
        assert_eq!(client.get_listing_count(), 1);

        let listing = client.get_listing(&id).unwrap();
        assert_eq!(listing.price, 50_000_000);
        assert_eq!(listing.status, ListingStatus::Active);
    }

    #[test]
    fn test_unlist_relist() {
        let (env, admin, vault, seller, _buyer) = setup_env();
        let contract_id = env.register(SportMarketplace, ());
        let client      = SportMarketplaceClient::new(&env, &contract_id);
        let token       = create_token(&env, &admin);

        client.initialize(&admin, &0, &vault);

        let id = client.list_item(
            &seller,
            &String::from_str(&env, "Tennis Racket"),
            &String::from_str(&env, "Wilson Pro Staff"),
            &SportCategory::Tennis,
            &ItemCondition::Good,
            &30_000_000,
            &token,
        );

        client.unlist_item(&seller, &id);
        let l = client.get_listing(&id).unwrap();
        assert_eq!(l.status, ListingStatus::Unlisted);

        client.relist_item(&seller, &id, &Some(25_000_000_i128));
        let l2 = client.get_listing(&id).unwrap();
        assert_eq!(l2.status, ListingStatus::Active);
        assert_eq!(l2.price, 25_000_000);
    }

    #[test]
    fn test_buy_item() {
        let (env, admin, vault, seller, buyer) = setup_env();
        let contract_id = env.register(SportMarketplace, ());
        let client      = SportMarketplaceClient::new(&env, &contract_id);
        let token_addr  = create_token(&env, &admin);

        // Mint tokens to buyer
        let token = TokenClient::new(&env, &token_addr);
        token.mint(&buyer, &100_000_000);

        client.initialize(&admin, &250, &vault);  // 2.5 % fee

        let id = client.list_item(
            &seller,
            &String::from_str(&env, "Basketball"),
            &String::from_str(&env, "Spalding NBA Official"),
            &SportCategory::Basketball,
            &ItemCondition::New,
            &40_000_000,
            &token_addr,
        );

        let seller_before = token.balance(&seller);
        let vault_before  = token.balance(&vault);

        client.buy_item(&buyer, &id);

        // 2.5 % of 40_000_000 = 1_000_000
        assert_eq!(token.balance(&seller), seller_before + 39_000_000);
        assert_eq!(token.balance(&vault),  vault_before  +  1_000_000);
        assert_eq!(token.balance(&buyer),  60_000_000);

        let listing = client.get_listing(&id).unwrap();
        assert_eq!(listing.status, ListingStatus::Sold);
    }

    #[test]
    fn test_filter_by_category() {
        let (env, admin, vault, seller, _buyer) = setup_env();
        let contract_id = env.register(SportMarketplace, ());
        let client      = SportMarketplaceClient::new(&env, &contract_id);
        let token       = create_token(&env, &admin);

        client.initialize(&admin, &0, &vault);

        client.list_item(&seller, &String::from_str(&env, "Ball"), &String::from_str(&env, ""),
            &SportCategory::Football, &ItemCondition::New, &1_000_000, &token);
        client.list_item(&seller, &String::from_str(&env, "Hoop"), &String::from_str(&env, ""),
            &SportCategory::Basketball, &ItemCondition::New, &2_000_000, &token);
        client.list_item(&seller, &String::from_str(&env, "Cleat"), &String::from_str(&env, ""),
            &SportCategory::Football, &ItemCondition::Good, &500_000, &token);

        let football = client.get_listings_by_category(&SportCategory::Football);
        assert_eq!(football.len(), 2);

        let basketball = client.get_listings_by_category(&SportCategory::Basketball);
        assert_eq!(basketball.len(), 1);
    }
}