#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};
use noether_common::{Asset, Error, OracleTrait, PriceData};

// ============================================================================
// Storage Keys
// ============================================================================

const ADMIN: Symbol = symbol_short!("ADMIN");
const INIT: Symbol = symbol_short!("INIT");

/// Storage key for asset prices
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Price(Asset),
    Timestamp(Asset),
}

// ============================================================================
// Mock Oracle Contract
// ============================================================================

/// Mock Oracle for local testing and development
/// Allows admin to set arbitrary prices for testing purposes
#[contract]
pub struct MockOracle;

#[contractimpl]
impl MockOracle {
    /// Initialize the mock oracle with an admin address
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&INIT) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&INIT, &true);

        // Set default prices (7 decimals)
        // XLM: $0.15
        env.storage().persistent().set(&DataKey::Price(Asset::Stellar), &1_500_000_i128);
        env.storage().persistent().set(&DataKey::Timestamp(Asset::Stellar), &env.ledger().timestamp());
        
        // USDC: $1.00
        env.storage().persistent().set(&DataKey::Price(Asset::USDC), &10_000_000_i128);
        env.storage().persistent().set(&DataKey::Timestamp(Asset::USDC), &env.ledger().timestamp());

        Ok(())
    }

    /// Set price for an asset (admin only)
    /// Price should be in 7 decimal format (e.g., 1_500_000 = $0.15)
    pub fn set_price(env: Env, asset: Asset, price: i128) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        env.storage().persistent().set(&DataKey::Price(asset.clone()), &price);
        env.storage().persistent().set(&DataKey::Timestamp(asset), &env.ledger().timestamp());

        Ok(())
    }

    /// Set prices for multiple assets at once (admin only)
    pub fn set_prices(env: Env, xlm_price: i128, usdc_price: i128) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let timestamp = env.ledger().timestamp();

        env.storage().persistent().set(&DataKey::Price(Asset::Stellar), &xlm_price);
        env.storage().persistent().set(&DataKey::Timestamp(Asset::Stellar), &timestamp);
        
        env.storage().persistent().set(&DataKey::Price(Asset::USDC), &usdc_price);
        env.storage().persistent().set(&DataKey::Timestamp(Asset::USDC), &timestamp);

        Ok(())
    }

    /// Get the current price for an asset
    pub fn get_price(env: Env, asset: Asset) -> Result<i128, Error> {
        env.storage().persistent().get(&DataKey::Price(asset))
            .ok_or(Error::AssetNotSupported)
    }

    /// Get price data with timestamp
    pub fn get_price_data(env: Env, asset: Asset) -> Result<PriceData, Error> {
        let price: i128 = env.storage().persistent().get(&DataKey::Price(asset.clone()))
            .ok_or(Error::AssetNotSupported)?;
        let timestamp: u64 = env.storage().persistent().get(&DataKey::Timestamp(asset))
            .ok_or(Error::AssetNotSupported)?;

        Ok(PriceData { price, timestamp })
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        env.storage().instance().get(&ADMIN)
            .ok_or(Error::NotInitialized)
    }

    /// Transfer admin to new address
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        env.storage().instance().set(&ADMIN, &new_admin);
        Ok(())
    }
}

// ============================================================================
// OracleTrait Implementation
// ============================================================================

impl OracleTrait for MockOracle {
    fn get_price(env: Env, asset: Asset) -> Result<i128, Error> {
        MockOracle::get_price(env, asset)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MockOracle);
        let client = MockOracleClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.get_admin(), admin);
    }

    #[test]
    fn test_default_prices() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MockOracle);
        let client = MockOracleClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        // Check default XLM price ($0.15 = 1_500_000 with 7 decimals)
        let xlm_price = client.get_price(&Asset::Stellar);
        assert_eq!(xlm_price, 1_500_000);

        // Check default USDC price ($1.00 = 10_000_000 with 7 decimals)
        let usdc_price = client.get_price(&Asset::USDC);
        assert_eq!(usdc_price, 10_000_000);
    }

    #[test]
    fn test_set_price() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, MockOracle);
        let client = MockOracleClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        // Set new XLM price ($0.20 = 2_000_000)
        client.set_price(&Asset::Stellar, &2_000_000);

        let xlm_price = client.get_price(&Asset::Stellar);
        assert_eq!(xlm_price, 2_000_000);
    }

    #[test]
    fn test_get_price_data() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MockOracle);
        let client = MockOracleClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        let price_data = client.get_price_data(&Asset::Stellar);
        assert_eq!(price_data.price, 1_500_000);
        assert!(price_data.timestamp > 0);
    }
}

