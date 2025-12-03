#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};
use noether_common::{Asset, Error, OracleTrait, PriceData};

// ============================================================================
// External Oracle Data Structures
// ============================================================================

/// Band Protocol reference data structure
#[derive(Clone, Debug)]
#[contracttype]
pub struct BandRefData {
    pub rate: u128,      // Price with 9 decimals
    pub last_updated: u64,
    pub request_id: u64,
}

/// DIA Protocol price data structure  
#[derive(Clone, Debug)]
#[contracttype]
pub struct DiaPriceData {
    pub price: u128,     // Price with 8 decimals
    pub timestamp: u64,
    pub symbol: Symbol,
}

// ============================================================================
// Storage Keys
// ============================================================================

const BAND_ADDR: Symbol = symbol_short!("BAND");
const DIA_ADDR: Symbol = symbol_short!("DIA");
const ADMIN: Symbol = symbol_short!("ADMIN");
const INIT: Symbol = symbol_short!("INIT");

// ============================================================================
// Constants
// ============================================================================

/// Standard precision for Soroban/Stellar (7 decimals)
const PRECISION: i128 = 10_000_000; // 1e7

/// Band Protocol uses 9 decimals
const BAND_DECIMALS: i128 = 1_000_000_000; // 1e9

/// DIA Protocol uses 8 decimals
const DIA_DECIMALS: i128 = 100_000_000; // 1e8

/// Maximum allowed staleness in seconds (60 seconds)
const MAX_STALENESS: u64 = 60;

/// Maximum allowed price deviation (1.5% = 150 basis points)
const MAX_DEVIATION_BPS: i128 = 150;

// ============================================================================
// External Contract Client Interfaces
// Note: These are interface definitions for cross-contract calls.
// In production, you would use contractimport! with actual WASM files.
// ============================================================================

// Band Oracle Interface:
// pub trait BandOracleClient {
//     fn get_reference_data(env: &Env, symbols: Vec<Symbol>) -> BandRefData;
// }

// DIA Oracle Interface:
// pub trait DiaOracleClient {
//     fn get_value(env: &Env, key: Symbol) -> DiaPriceData;
// }

// ============================================================================
// Oracle Adapter Contract
// ============================================================================

#[contract]
pub struct OracleAdapter;

#[contractimpl]
impl OracleAdapter {
    /// Initialize the oracle adapter with Band and DIA contract addresses
    pub fn initialize(env: Env, admin: Address, band: Address, dia: Address) -> Result<(), Error> {
        // Check if already initialized
        if env.storage().instance().has(&INIT) {
            return Err(Error::AlreadyInitialized);
        }

        // Store addresses
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&BAND_ADDR, &band);
        env.storage().instance().set(&DIA_ADDR, &dia);
        env.storage().instance().set(&INIT, &true);

        Ok(())
    }

    /// Update oracle addresses (admin only)
    pub fn update_oracles(env: Env, band: Address, dia: Address) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&ADMIN)
            .ok_or(Error::NotInitialized)?;
        admin.require_auth();

        env.storage().instance().set(&BAND_ADDR, &band);
        env.storage().instance().set(&DIA_ADDR, &dia);

        Ok(())
    }

    /// Get the current price for an asset with safety checks
    pub fn get_price(env: Env, asset: Asset) -> Result<i128, Error> {
        // Get oracle addresses
        let _band_addr: Address = env.storage().instance().get(&BAND_ADDR)
            .ok_or(Error::NotInitialized)?;
        let _dia_addr: Address = env.storage().instance().get(&DIA_ADDR)
            .ok_or(Error::NotInitialized)?;

        // Map asset to oracle symbols
        let (band_symbol, dia_key) = Self::asset_to_symbols(&env, &asset)?;

        // Fetch prices from both oracles
        // Note: In production, these would be actual cross-contract calls
        // For now, we simulate the logic structure
        let band_price = Self::fetch_band_price(&env, &band_symbol)?;
        let dia_price = Self::fetch_dia_price(&env, &dia_key)?;

        // Check for price deviation
        Self::check_deviation(band_price, dia_price)?;

        // Return average of both prices
        let avg_price = (band_price + dia_price) / 2;
        Ok(avg_price)
    }

    /// Get price data with timestamp
    pub fn get_price_data(env: Env, asset: Asset) -> Result<PriceData, Error> {
        let price = Self::get_price(env.clone(), asset)?;
        Ok(PriceData {
            price,
            timestamp: env.ledger().timestamp(),
        })
    }

    /// Get Band oracle address
    pub fn get_band_address(env: Env) -> Result<Address, Error> {
        env.storage().instance().get(&BAND_ADDR)
            .ok_or(Error::NotInitialized)
    }

    /// Get DIA oracle address
    pub fn get_dia_address(env: Env) -> Result<Address, Error> {
        env.storage().instance().get(&DIA_ADDR)
            .ok_or(Error::NotInitialized)
    }
}

// ============================================================================
// Internal Implementation
// ============================================================================

impl OracleAdapter {
    /// Map Asset enum to oracle-specific symbols
    fn asset_to_symbols(env: &Env, asset: &Asset) -> Result<(Symbol, Symbol), Error> {
        match asset {
            Asset::Stellar => Ok((
                Symbol::new(env, "XLM"),
                Symbol::new(env, "XLM/USD"),
            )),
            Asset::USDC => Ok((
                Symbol::new(env, "USDC"),
                Symbol::new(env, "USDC/USD"),
            )),
        }
    }

    /// Fetch price from Band Protocol
    /// Normalizes to 7 decimals (Stellar standard)
    fn fetch_band_price(env: &Env, _symbol: &Symbol) -> Result<i128, Error> {
        // In production, this would be:
        // let band_addr: Address = env.storage().instance().get(&BAND_ADDR).unwrap();
        // let client = band_client::Client::new(env, &band_addr);
        // let symbols = Vec::from_array(env, [symbol.clone()]);
        // let ref_data = client.get_reference_data(&symbols);
        
        // Simulated response for structure demonstration
        // This would be replaced with actual cross-contract call
        let simulated_rate: u128 = 150_000_000; // 0.15 USD with 9 decimals (XLM price)
        let simulated_timestamp: u64 = env.ledger().timestamp();

        // Check staleness
        let current_time = env.ledger().timestamp();
        if simulated_timestamp < current_time.saturating_sub(MAX_STALENESS) {
            return Err(Error::OracleStale);
        }

        // Normalize from 9 decimals to 7 decimals
        let normalized_price = (simulated_rate as i128 * PRECISION) / BAND_DECIMALS;
        Ok(normalized_price)
    }

    /// Fetch price from DIA Protocol
    /// Normalizes to 7 decimals (Stellar standard)
    fn fetch_dia_price(env: &Env, _key: &Symbol) -> Result<i128, Error> {
        // In production, this would be:
        // let dia_addr: Address = env.storage().instance().get(&DIA_ADDR).unwrap();
        // let client = dia_client::Client::new(env, &dia_addr);
        // let price_data = client.get_value(key);
        
        // Simulated response for structure demonstration
        let simulated_price: u128 = 15_000_000; // 0.15 USD with 8 decimals
        let simulated_timestamp: u64 = env.ledger().timestamp();

        // Check staleness
        let current_time = env.ledger().timestamp();
        if simulated_timestamp < current_time.saturating_sub(MAX_STALENESS) {
            return Err(Error::OracleStale);
        }

        // Normalize from 8 decimals to 7 decimals
        let normalized_price = (simulated_price as i128 * PRECISION) / DIA_DECIMALS;
        Ok(normalized_price)
    }

    /// Check if price deviation between oracles exceeds threshold
    fn check_deviation(price_a: i128, price_b: i128) -> Result<(), Error> {
        if price_a == 0 || price_b == 0 {
            return Err(Error::PriceDivergence);
        }

        // Calculate absolute difference
        let diff = if price_a > price_b {
            price_a - price_b
        } else {
            price_b - price_a
        };

        // Calculate deviation in basis points (1 bp = 0.01%)
        // deviation_bps = (diff * 10000) / price_a
        let deviation_bps = (diff * 10_000) / price_a;

        if deviation_bps > MAX_DEVIATION_BPS {
            return Err(Error::PriceDivergence);
        }

        Ok(())
    }
}

// ============================================================================
// OracleTrait Implementation
// ============================================================================

impl OracleTrait for OracleAdapter {
    fn get_price(env: Env, asset: Asset) -> Result<i128, Error> {
        OracleAdapter::get_price(env, asset)
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
        let contract_id = env.register_contract(None, OracleAdapter);
        let client = OracleAdapterClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let band = Address::generate(&env);
        let dia = Address::generate(&env);

        client.initialize(&admin, &band, &dia);

        assert_eq!(client.get_band_address(), band);
        assert_eq!(client.get_dia_address(), dia);
    }

    #[test]
    fn test_check_deviation_within_limit() {
        let price_a: i128 = 1_000_000; // 0.1 with 7 decimals
        let price_b: i128 = 1_010_000; // 0.101 with 7 decimals (1% diff)
        
        assert!(OracleAdapter::check_deviation(price_a, price_b).is_ok());
    }

    #[test]
    fn test_check_deviation_exceeds_limit() {
        let price_a: i128 = 1_000_000; // 0.1 with 7 decimals
        let price_b: i128 = 1_020_000; // 0.102 with 7 decimals (2% diff)
        
        assert!(OracleAdapter::check_deviation(price_a, price_b).is_err());
    }
}

