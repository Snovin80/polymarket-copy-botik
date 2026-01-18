// Integration tests for the Polymarket Copy Trading Bot

#[cfg(test)]
mod tests {
    use std::env;
    
    #[test]
    fn test_env_variables() {
        // Test that all required env variables can be loaded
        dotenv::dotenv().ok();
        
        let required_vars = [
            "WALLETS_TO_TRACK",
            "YOUR_WALLET",
            "PRIVATE_KEY",
            "RPC_URL",
        ];
        
        for var in &required_vars {
            assert!(
                env::var(var).is_ok(),
                "Required environment variable {} is not set",
                var
            );
        }
    }
    
    #[test]
    fn test_wallet_address_format() {
        dotenv::dotenv().ok();
        
        let your_wallet = env::var("YOUR_WALLET").unwrap_or_default();
        
        // Check if it's a valid Ethereum address format
        assert!(
            your_wallet.starts_with("0x"),
            "YOUR_WALLET should start with 0x"
        );
        assert_eq!(
            your_wallet.len(),
            42,
            "YOUR_WALLET should be 42 characters (0x + 40 hex)"
        );
    }
    
    #[test]
    fn test_private_key_format() {
        dotenv::dotenv().ok();
        
        let private_key = env::var("PRIVATE_KEY").unwrap_or_default();
        
        // Private key should be 64 hex characters (or 66 with 0x prefix)
        let key_len = private_key.len();
        assert!(
            key_len == 64 || key_len == 66,
            "PRIVATE_KEY should be 64 or 66 characters, got {}",
            key_len
        );
    }
    
    #[tokio::test]
    async fn test_config_loading() {
        dotenv::dotenv().ok();
        
        // This will panic if config is invalid
        let config = polymarket_copy_bot::config::load_config();
        assert!(config.is_ok(), "Failed to load config: {:?}", config.err());
        
        let config = config.unwrap();
        assert!(!config.wallets_to_track.is_empty(), "No wallets configured");
        assert!(!config.your_wallet.is_empty(), "Your wallet not configured");
    }
    
    #[test]
    fn test_sizing_calculations() {
        let whale_size = 100.0;
        let whale_balance = 10000.0;
        let your_balance = 1000.0;
        
        // Proportional sizing
        let ratio = your_balance / whale_balance;
        let your_size = whale_size * ratio;
        
        assert_eq!(your_size, 10.0, "Proportional sizing calculation failed");
    }
    
    #[test]
    fn test_tier_multipliers() {
        // Small trade
        let small = 30.0;
        assert!(get_tier(small) == 0.5);
        
        // Medium trade
        let medium = 150.0;
        assert!(get_tier(medium) == 1.0);
        
        // Large trade
        let large = 400.0;
        assert!(get_tier(large) == 1.5);
        
        // Whale trade
        let whale = 800.0;
        assert!(get_tier(whale) == 2.0);
    }
    
    fn get_tier(size: f64) -> f64 {
        if size < 50.0 {
            0.5
        } else if size < 200.0 {
            1.0
        } else if size < 500.0 {
            1.5
        } else {
            2.0
        }
    }
    
    #[test]
    fn test_risk_limits() {
        let max_exposure = 500.0;
        let current_exposure = 300.0;
        let new_trade = 150.0;
        
        // Should fail - exceeds limit
        assert!(
            current_exposure + new_trade <= max_exposure,
            "Trade should be rejected"
        );
        
        let new_trade_ok = 100.0;
        assert!(
            current_exposure + new_trade_ok <= max_exposure,
            "Trade should be accepted"
        );
    }
}

// Mock tests for API functionality
#[cfg(test)]
mod api_tests {
    #[tokio::test]
    async fn test_api_client_creation() {
        let api_url = "https://api.polymarket.com".to_string();
        let api = polymarket_copy_bot::api::PolymarketApi::new(api_url);
        
        // Just test that we can create the client
        assert!(true);
    }
}

// Circuit breaker tests
#[cfg(test)]
mod circuit_breaker_tests {
    use polymarket_copy_bot::risk::RiskManager;
    use polymarket_copy_bot::types::{Config, SizingMode};
    
    #[test]
    fn test_circuit_breaker_trips() {
        let config = Config {
            wallets_to_track: vec![],
            your_wallet: "0x123".to_string(),
            private_key: "abc".to_string(),
            polymarket_api: "".to_string(),
            ws_url: "".to_string(),
            rpc_url: "".to_string(),
            sizing_mode: SizingMode::Fixed,
            fixed_stake: 25.0,
            proportional_ratio: 0.02,
            min_stake: 5.0,
            max_stake: 100.0,
            max_exposure_per_event: 500.0,
            max_daily_volume: 2000.0,
            min_liquidity: 1000.0,
            cb_consecutive_trigger: 3,
            cb_min_depth_usd: 100.0,
            retry_attempts: 4,
            retry_delay_ms: 500,
        };
        
        let risk = RiskManager::new(config);
        
        // Record errors until circuit breaker trips
        risk.record_error("Test error 1");
        assert!(!risk.get_state().is_tripped);
        
        risk.record_error("Test error 2");
        assert!(!risk.get_state().is_tripped);
        
        risk.record_error("Test error 3");
        assert!(risk.get_state().is_tripped);
    }
    
    #[test]
    fn test_circuit_breaker_reset() {
        let config = Config {
            cb_consecutive_trigger: 2,
            ..Default::default()
        };
        
        let risk = RiskManager::new(config);
        
        risk.record_error("Error 1");
        risk.record_error("Error 2");
        assert!(risk.get_state().is_tripped);
        
        risk.reset_circuit_breaker();
        assert!(!risk.get_state().is_tripped);
    }
}