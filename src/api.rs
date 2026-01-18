use crate::types::{Market, Trade, OrderRequest, OrderResponse, TradeSide};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;

pub struct PolymarketApi {
    client: Client,
    base_url: String,
}

impl PolymarketApi {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
    
    pub async fn get_market(&self, market_id: &str) -> Result<Market> {
        let url = format!("{}/markets/{}", self.base_url, market_id);
        let resp = self.client.get(&url)
            .send()
            .await
            .context("Failed to fetch market")?
            .json::<serde_json::Value>()
            .await?;
        
        Ok(Market {
            id: market_id.to_string(),
            event_id: resp["event_id"].as_str().unwrap_or("").to_string(),
            question: resp["question"].as_str().unwrap_or("").to_string(),
            yes_price: resp["yes_price"].as_f64().unwrap_or(0.5),
            no_price: resp["no_price"].as_f64().unwrap_or(0.5),
            liquidity: resp["liquidity"].as_f64().unwrap_or(0.0),
            volume_24h: resp["volume_24h"].as_f64().unwrap_or(0.0),
        })
    }
    
    pub async fn get_trades(&self, wallet: &str, since: i64) -> Result<Vec<Trade>> {
        let url = format!("{}/trades", self.base_url);
        let resp = self.client.get(&url)
            .query(&[("wallet", wallet), ("since", &since.to_string())])
            .send()
            .await
            .context("Failed to fetch trades")?
            .json::<Vec<serde_json::Value>>()
            .await?;
        
        let mut trades = Vec::new();
        for item in resp {
            trades.push(Trade {
                wallet: item["wallet"].as_str().unwrap_or("").to_string(),
                event_id: item["event_id"].as_str().unwrap_or("").to_string(),
                market_id: item["market_id"].as_str().unwrap_or("").to_string(),
                side: if item["side"].as_str() == Some("BUY") {
                    TradeSide::BUY
                } else {
                    TradeSide::SELL
                },
                shares: item["shares"].as_f64().unwrap_or(0.0),
                price: item["price"].as_f64().unwrap_or(0.0),
                timestamp: item["timestamp"].as_i64().unwrap_or(0),
                tx_hash: item["tx_hash"].as_str().map(|s| s.to_string()),
            });
        }
        
        Ok(trades)
    }
    
    pub async fn get_orderbook(&self, market_id: &str) -> Result<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        let url = format!("{}/orderbook/{}", self.base_url, market_id);
        let resp = self.client.get(&url)
            .send()
            .await
            .context("Failed to fetch orderbook")?
            .json::<serde_json::Value>()
            .await?;
        
        let bids = resp["bids"].as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        let price = v["price"].as_f64()?;
                        let size = v["size"].as_f64()?;
                        Some((price, size))
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        let asks = resp["asks"].as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| {
                        let price = v["price"].as_f64()?;
                        let size = v["size"].as_f64()?;
                        Some((price, size))
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        Ok((bids, asks))
    }
    
    pub async fn place_order(&self, req: OrderRequest, api_key: &str) -> Result<OrderResponse> {
        let url = format!("{}/orders", self.base_url);
        
        let body = json!({
            "market_id": req.market_id,
            "side": match req.side {
                TradeSide::BUY => "BUY",
                TradeSide::SELL => "SELL",
            },
            "shares": req.shares,
            "price": req.price,
            "type": format!("{:?}", req.order_type),
        });
        
        let resp = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await
            .context("Failed to place order")?
            .json::<serde_json::Value>()
            .await?;
        
        Ok(OrderResponse {
            order_id: resp["order_id"].as_str().unwrap_or("").to_string(),
            status: resp["status"].as_str().unwrap_or("").to_string(),
            filled_shares: resp["filled_shares"].as_f64().unwrap_or(0.0),
            avg_fill_price: resp["avg_fill_price"].as_f64().unwrap_or(0.0),
        })
    }
    
    pub async fn get_balance(&self, wallet: &str) -> Result<f64> {
        let url = format!("{}/balance/{}", self.base_url, wallet);
        let resp = self.client.get(&url)
            .send()
            .await
            .context("Failed to fetch balance")?
            .json::<serde_json::Value>()
            .await?;
        
        Ok(resp["balance"].as_f64().unwrap_or(0.0))
    }
}