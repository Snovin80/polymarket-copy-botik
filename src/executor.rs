use crate::api::PolymarketApi;
use crate::types::{Config, Trade, TradeSide, OrderRequest, OrderType, OrderResponse};
use anyhow::{Context, Result};
use std::time::Duration;

pub struct TradeExecutor {
    api: PolymarketApi,
    config: Config,
}

impl TradeExecutor {
    pub fn new(api: PolymarketApi, config: Config) -> Self {
        Self { api, config }
    }
    
    pub async fn execute_trade(&self, trade: &Trade, shares: f64) -> Result<OrderResponse> {
        let order_type = match trade.side {
            TradeSide::BUY => OrderType::FAK,  // Fill-And-Kill for buys
            TradeSide::SELL => OrderType::GTD,  // Good-Till-Date for sells
        };
        
        let order = OrderRequest {
            market_id: trade.market_id.clone(),
            side: trade.side.clone(),
            shares,
            price: Some(trade.price),
            order_type,
        };
        
        let result = self.execute_with_retry(order).await;
        
        match &result {
            Ok(resp) => {
                tracing::info!(
                    "Trade executed: {} {:.2} shares @ ${:.4} (order_id: {})",
                    match trade.side {
                        TradeSide::BUY => "BUY",
                        TradeSide::SELL => "SELL",
                    },
                    resp.filled_shares,
                    resp.avg_fill_price,
                    resp.order_id
                );
            }
            Err(e) => {
                tracing::error!("Trade execution failed: {}", e);
            }
        }
        
        result
    }
    
    async fn execute_with_retry(&self, order: OrderRequest) -> Result<OrderResponse> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.config.retry_attempts {
            attempts += 1;
            
            match self.api.place_order(order.clone(), &self.config.private_key).await {
                Ok(resp) => {
                    if resp.status == "filled" || resp.status == "partially_filled" {
                        return Ok(resp);
                    }
                    
                    if resp.status == "cancelled" || resp.status == "rejected" {
                        anyhow::bail!("Order {} by exchange: {}", resp.status, resp.order_id);
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempts < self.config.retry_attempts {
                        tracing::warn!(
                            "Attempt {}/{} failed, retrying in {}ms...",
                            attempts,
                            self.config.retry_attempts,
                            self.config.retry_delay_ms
                        );
                        
                        tokio::time::sleep(Duration::from_millis(
                            self.config.retry_delay_ms * (attempts as u64)
                        )).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap().context(format!(
            "Failed to execute order after {} attempts",
            self.config.retry_attempts
        )))
    }
    
    pub async fn execute_market_order(&self, trade: &Trade, usd_amount: f64) -> Result<OrderResponse> {
        let shares = if trade.price > 0.0 {
            usd_amount / trade.price
        } else {
            anyhow::bail!("Invalid price: {}", trade.price);
        };
        
        let order = OrderRequest {
            market_id: trade.market_id.clone(),
            side: trade.side.clone(),
            shares,
            price: None,  // Market order
            order_type: OrderType::MARKET,
        };
        
        self.execute_with_retry(order).await
    }
    
    pub async fn close_position(&self, market_id: &str, shares: f64, side: TradeSide) -> Result<OrderResponse> {
        // To close a BUY position, we SELL
        // To close a SELL position, we BUY
        let close_side = match side {
            TradeSide::BUY => TradeSide::SELL,
            TradeSide::SELL => TradeSide::BUY,
        };
        
        let order = OrderRequest {
            market_id: market_id.to_string(),
            side: close_side,
            shares,
            price: None,
            order_type: OrderType::MARKET,
        };
        
        tracing::info!("Closing position: {} {:.2} shares on {}", 
            match side {
                TradeSide::BUY => "SELL",
                TradeSide::SELL => "BUY",
            },
            shares, 
            market_id
        );
        
        self.execute_with_retry(order).await
    }
    
    pub async fn get_estimated_price(&self, market_id: &str, side: &TradeSide) -> Result<f64> {
        let (bids, asks) = self.api.get_orderbook(market_id).await?;
        
        let price = match side {
            TradeSide::BUY => {
                // For buying, we look at asks (sellers)
                asks.first().map(|(p, _)| *p).unwrap_or(0.5)
            }
            TradeSide::SELL => {
                // For selling, we look at bids (buyers)
                bids.first().map(|(p, _)| *p).unwrap_or(0.5)
            }
        };
        
        Ok(price)
    }
}