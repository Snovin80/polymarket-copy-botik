use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Default for Config {
    fn default() -> Self {
        Self {
            wallets_to_track: vec![],
            your_wallet: String::new(),
            private_key: String::new(),
            polymarket_api: String::new(),
            ws_url: String::new(),
            rpc_url: String::new(),
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub wallet: String,
    pub event_id: String,
    pub market_id: String,
    pub side: TradeSide,
    pub shares: f64,
    pub price: f64,
    pub timestamp: i64,
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeSide {
    BUY,
    SELL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub id: String,
    pub event_id: String,
    pub question: String,
    pub yes_price: f64,
    pub no_price: f64,
    pub liquidity: f64,
    pub volume_24h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub market_id: String,
    pub side: TradeSide,
    pub shares: f64,
    pub avg_price: f64,
    pub current_price: f64,
    pub pnl: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub market_id: String,
    pub side: TradeSide,
    pub shares: f64,
    pub price: Option<f64>,
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    MARKET,
    LIMIT,
    FAK,  // Fill-And-Kill
    GTD,  // Good-Till-Date
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
    pub filled_shares: f64,
    pub avg_fill_price: f64,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    pub consecutive_errors: u32,
    pub total_trades_today: u32,
    pub total_volume_today: f64,
    pub is_tripped: bool,
    pub trip_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub wallets_to_track: Vec<String>,
    pub your_wallet: String,
    pub private_key: String,
    pub polymarket_api: String,
    pub ws_url: String,
    pub rpc_url: String,
    
    // Sizing
    pub sizing_mode: SizingMode,
    pub fixed_stake: f64,
    pub proportional_ratio: f64,
    pub min_stake: f64,
    pub max_stake: f64,
    
    // Risk
    pub max_exposure_per_event: f64,
    pub max_daily_volume: f64,
    pub min_liquidity: f64,
    pub cb_consecutive_trigger: u32,
    pub cb_min_depth_usd: f64,
    
    // Execution
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SizingMode {
    Fixed,
    Proportional,
    TierBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: i64,
}