use crate::types::{Config, SizingMode};
use anyhow::{Context, Result};
use std::env;

pub fn load_config() -> Result<Config> {
    dotenv::dotenv().ok();
    
    let wallets = env::var("WALLETS_TO_TRACK")
        .context("WALLETS_TO_TRACK not set")?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    
    let sizing_mode = match env::var("SIZING_MODE")
        .unwrap_or_else(|_| "fixed".to_string())
        .to_lowercase()
        .as_str()
    {
        "proportional" => SizingMode::Proportional,
        "tier" | "tierbased" => SizingMode::TierBased,
        _ => SizingMode::Fixed,
    };
    
    Ok(Config {
        wallets_to_track: wallets,
        your_wallet: env::var("YOUR_WALLET")
            .context("YOUR_WALLET not set")?,
        private_key: env::var("PRIVATE_KEY")
            .context("PRIVATE_KEY not set")?,
        polymarket_api: env::var("POLYMARKET_API")
            .unwrap_or_else(|_| "https://api.polymarket.com".to_string()),
        ws_url: env::var("WS_URL")
            .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws".to_string()),
        rpc_url: env::var("RPC_URL")
            .context("RPC_URL not set (use Alchemy/Infura)")?,
        
        sizing_mode,
        fixed_stake: env::var("FIXED_STAKE")
            .unwrap_or_else(|_| "25.0".to_string())
            .parse()?,
        proportional_ratio: env::var("PROPORTIONAL_RATIO")
            .unwrap_or_else(|_| "0.02".to_string())
            .parse()?,
        min_stake: env::var("MIN_STAKE")
            .unwrap_or_else(|_| "5.0".to_string())
            .parse()?,
        max_stake: env::var("MAX_STAKE")
            .unwrap_or_else(|_| "100.0".to_string())
            .parse()?,
        
        max_exposure_per_event: env::var("MAX_EXPOSURE_PER_EVENT")
            .unwrap_or_else(|_| "500.0".to_string())
            .parse()?,
        max_daily_volume: env::var("MAX_DAILY_VOLUME")
            .unwrap_or_else(|_| "2000.0".to_string())
            .parse()?,
        min_liquidity: env::var("MIN_LIQUIDITY")
            .unwrap_or_else(|_| "1000.0".to_string())
            .parse()?,
        cb_consecutive_trigger: env::var("CB_CONSECUTIVE_TRIGGER")
            .unwrap_or_else(|_| "3".to_string())
            .parse()?,
        cb_min_depth_usd: env::var("CB_MIN_DEPTH_USD")
            .unwrap_or_else(|_| "100.0".to_string())
            .parse()?,
        
        retry_attempts: env::var("RETRY_ATTEMPTS")
            .unwrap_or_else(|_| "4".to_string())
            .parse()?,
        retry_delay_ms: env::var("RETRY_DELAY_MS")
            .unwrap_or_else(|_| "500".to_string())
            .parse()?,
    })
}

pub fn validate_config(config: &Config) -> Result<()> {
    if config.wallets_to_track.is_empty() {
        anyhow::bail!("No wallets to track configured");
    }
    
    if config.your_wallet.is_empty() {
        anyhow::bail!("YOUR_WALLET not configured");
    }
    
    if config.private_key.is_empty() || config.private_key.len() < 64 {
        anyhow::bail!("Invalid PRIVATE_KEY");
    }
    
    if config.fixed_stake < config.min_stake {
        anyhow::bail!("FIXED_STAKE must be >= MIN_STAKE");
    }
    
    if config.max_stake < config.min_stake {
        anyhow::bail!("MAX_STAKE must be >= MIN_STAKE");
    }
    
    tracing::info!("Config validation passed");
    Ok(())
}