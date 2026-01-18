use crate::types::{Config, SizingMode, Trade};
use anyhow::Result;

pub struct PositionSizer {
    config: Config,
}

impl PositionSizer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub async fn calculate_size(&self, whale_trade: &Trade, your_balance: f64, whale_balance: f64) -> Result<f64> {
        let size = match self.config.sizing_mode {
            SizingMode::Fixed => self.config.fixed_stake,
            
            SizingMode::Proportional => {
                let ratio = your_balance / whale_balance.max(1.0);
                whale_trade.shares * whale_trade.price * ratio
            },
            
            SizingMode::TierBased => {
                let trade_size = whale_trade.shares * whale_trade.price;
                let multiplier = self.get_tier_multiplier(trade_size);
                whale_trade.shares * multiplier * self.config.proportional_ratio
            },
        };
        
        // Apply limits
        let size = size.max(self.config.min_stake);
        let size = size.min(self.config.max_stake);
        
        // Check if we have enough balance
        let size = size.min(your_balance * 0.95); // Keep 5% buffer
        
        tracing::info!(
            "Calculated size: ${:.2} (mode: {:?}, whale: ${:.2})",
            size,
            self.config.sizing_mode,
            whale_trade.shares * whale_trade.price
        );
        
        Ok(size)
    }
    
    fn get_tier_multiplier(&self, trade_size_usd: f64) -> f64 {
        // Tier-based multipliers
        // Small trades get lower weight, large trades get higher weight
        if trade_size_usd < 50.0 {
            0.5 // 50% weight for small trades
        } else if trade_size_usd < 200.0 {
            1.0 // 100% weight for medium trades
        } else if trade_size_usd < 500.0 {
            1.5 // 150% weight for large trades
        } else {
            2.0 // 200% weight for whale trades
        }
    }
    
    pub fn shares_from_usd(&self, usd_amount: f64, price: f64) -> f64 {
        if price <= 0.0 {
            return 0.0;
        }
        usd_amount / price
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TradeSide;
    
    #[tokio::test]
    async fn test_fixed_sizing() {
        let config = Config {
            sizing_mode: SizingMode::Fixed,
            fixed_stake: 25.0,
            min_stake: 5.0,
            max_stake: 100.0,
            ..Default::default()
        };
        
        let sizer = PositionSizer::new(config);
        let trade = Trade {
            wallet: "0xwhale".to_string(),
            event_id: "event1".to_string(),
            market_id: "market1".to_string(),
            side: TradeSide::BUY,
            shares: 100.0,
            price: 0.5,
            timestamp: 0,
            tx_hash: None,
        };
        
        let size = sizer.calculate_size(&trade, 1000.0, 10000.0).await.unwrap();
        assert_eq!(size, 25.0);
    }
    
    #[tokio::test]
    async fn test_proportional_sizing() {
        let config = Config {
            sizing_mode: SizingMode::Proportional,
            min_stake: 5.0,
            max_stake: 100.0,
            ..Default::default()
        };
        
        let sizer = PositionSizer::new(config);
        let trade = Trade {
            wallet: "0xwhale".to_string(),
            event_id: "event1".to_string(),
            market_id: "market1".to_string(),
            side: TradeSide::BUY,
            shares: 100.0,
            price: 0.5,
            timestamp: 0,
            tx_hash: None,
        };
        
        // Your balance is 10% of whale's balance
        // So you should trade 10% of whale's trade
        let size = sizer.calculate_size(&trade, 1000.0, 10000.0).await.unwrap();
        assert_eq!(size, 5.0); // 100 shares * 0.5 price * 0.1 ratio = 5
    }
}