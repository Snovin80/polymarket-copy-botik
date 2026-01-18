use crate::types::{Config, CircuitBreakerState, Trade, Market};
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct RiskManager {
    config: Config,
    state: Arc<Mutex<CircuitBreakerState>>,
    event_exposure: Arc<Mutex<HashMap<String, f64>>>,
}

impl RiskManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState {
                consecutive_errors: 0,
                total_trades_today: 0,
                total_volume_today: 0.0,
                is_tripped: false,
                trip_reason: None,
            })),
            event_exposure: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn check_can_trade(&self, trade: &Trade, market: &Market, size_usd: f64) -> Result<()> {
        // Check if circuit breaker is tripped
        {
            let state = self.state.lock().unwrap();
            if state.is_tripped {
                bail!("Circuit breaker tripped: {}", state.trip_reason.as_ref().unwrap_or(&"Unknown".to_string()));
            }
        }
        
        // Check daily volume limit
        {
            let state = self.state.lock().unwrap();
            if state.total_volume_today + size_usd > self.config.max_daily_volume {
                bail!("Daily volume limit exceeded: ${:.2} + ${:.2} > ${:.2}",
                    state.total_volume_today, size_usd, self.config.max_daily_volume);
            }
        }
        
        // Check event exposure limit
        {
            let mut exposure = self.event_exposure.lock().unwrap();
            let current_exposure = exposure.get(&trade.event_id).copied().unwrap_or(0.0);
            if current_exposure + size_usd > self.config.max_exposure_per_event {
                bail!("Event exposure limit exceeded: ${:.2} + ${:.2} > ${:.2}",
                    current_exposure, size_usd, self.config.max_exposure_per_event);
            }
        }
        
        // Check market liquidity
        if market.liquidity < self.config.min_liquidity {
            bail!("Insufficient liquidity: ${:.2} < ${:.2}",
                market.liquidity, self.config.min_liquidity);
        }
        
        // Check orderbook depth
        let depth_ok = market.liquidity >= self.config.cb_min_depth_usd;
        if !depth_ok {
            bail!("Orderbook depth too low: ${:.2} < ${:.2}",
                market.liquidity, self.config.cb_min_depth_usd);
        }
        
        tracing::info!("Risk checks passed for trade on {}", trade.market_id);
        Ok(())
    }
    
    pub fn record_trade(&self, trade: &Trade, size_usd: f64) {
        let mut state = self.state.lock().unwrap();
        state.total_trades_today += 1;
        state.total_volume_today += size_usd;
        state.consecutive_errors = 0; // Reset on successful trade
        
        let mut exposure = self.event_exposure.lock().unwrap();
        *exposure.entry(trade.event_id.clone()).or_insert(0.0) += size_usd;
        
        tracing::info!(
            "Trade recorded: #{} today, ${:.2} volume, ${:.2} event exposure",
            state.total_trades_today,
            state.total_volume_today,
            exposure.get(&trade.event_id).unwrap_or(&0.0)
        );
    }
    
    pub fn record_error(&self, error: &str) {
        let mut state = self.state.lock().unwrap();
        state.consecutive_errors += 1;
        
        tracing::warn!("Error recorded: {} (consecutive: {})", error, state.consecutive_errors);
        
        if state.consecutive_errors >= self.config.cb_consecutive_trigger {
            state.is_tripped = true;
            state.trip_reason = Some(format!("Too many consecutive errors: {}", state.consecutive_errors));
            tracing::error!("CIRCUIT BREAKER TRIPPED: {}", state.trip_reason.as_ref().unwrap());
        }
    }
    
    pub fn reset_circuit_breaker(&self) {
        let mut state = self.state.lock().unwrap();
        state.is_tripped = false;
        state.consecutive_errors = 0;
        state.trip_reason = None;
        tracing::info!("Circuit breaker reset");
    }
    
    pub fn reset_daily_stats(&self) {
        let mut state = self.state.lock().unwrap();
        state.total_trades_today = 0;
        state.total_volume_today = 0.0;
        
        let mut exposure = self.event_exposure.lock().unwrap();
        exposure.clear();
        
        tracing::info!("Daily stats reset");
    }
    
    pub fn get_state(&self) -> CircuitBreakerState {
        self.state.lock().unwrap().clone()
    }
    
    pub fn is_whale_verified(&self, wallet: &str) -> bool {
        // Check if wallet is in our tracked list
        self.config.wallets_to_track.contains(&wallet.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{TradeSide, SizingMode};
    
    #[test]
    fn test_circuit_breaker() {
        let config = Config {
            cb_consecutive_trigger: 3,
            max_daily_volume: 1000.0,
            max_exposure_per_event: 500.0,
            min_liquidity: 100.0,
            cb_min_depth_usd: 50.0,
            wallets_to_track: vec!["0xwhale".to_string()],
            ..Default::default()
        };
        
        let risk = RiskManager::new(config);
        
        // First error
        risk.record_error("Test error 1");
        assert!(!risk.get_state().is_tripped);
        
        // Second error
        risk.record_error("Test error 2");
        assert!(!risk.get_state().is_tripped);
        
        // Third error - should trip
        risk.record_error("Test error 3");
        assert!(risk.get_state().is_tripped);
        
        // Reset
        risk.reset_circuit_breaker();
        assert!(!risk.get_state().is_tripped);
    }
}