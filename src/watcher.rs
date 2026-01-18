use crate::types::{Trade, TradeSide, WebSocketEvent};
use anyhow::{Context, Result};
use async_channel::{Sender, Receiver, bounded};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub struct WalletWatcher {
    ws_url: String,
    wallets: Vec<String>,
}

impl WalletWatcher {
    pub fn new(ws_url: String, wallets: Vec<String>) -> Self {
        Self { ws_url, wallets }
    }
    
    pub async fn start(&self) -> Result<Receiver<Trade>> {
        let (tx, rx) = bounded(1000);
        
        for wallet in &self.wallets {
            let wallet_clone = wallet.clone();
            let ws_url = self.ws_url.clone();
            let tx_clone = tx.clone();
            
            tokio::spawn(async move {
                if let Err(e) = watch_wallet(ws_url, wallet_clone, tx_clone).await {
                    tracing::error!("Wallet watcher error: {}", e);
                }
            });
        }
        
        Ok(rx)
    }
}

async fn watch_wallet(ws_url: String, wallet: String, tx: Sender<Trade>) -> Result<()> {
    loop {
        match connect_and_watch(&ws_url, &wallet, &tx).await {
            Ok(_) => tracing::info!("WebSocket connection closed for {}", wallet),
            Err(e) => {
                tracing::error!("WebSocket error for {}: {}", wallet, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn connect_and_watch(ws_url: &str, wallet: &str, tx: &Sender<Trade>) -> Result<()> {
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .context("Failed to connect to WebSocket")?;
    
    let (mut write, mut read) = ws_stream.split();
    
    // Subscribe to wallet trades
    let subscribe_msg = json!({
        "type": "subscribe",
        "channel": "trades",
        "wallet": wallet,
    });
    
    write.send(Message::Text(subscribe_msg.to_string()))
        .await
        .context("Failed to send subscribe message")?;
    
    tracing::info!("Subscribed to trades for wallet: {}", wallet);
    
    // Keep connection alive
    let wallet_clone = wallet.to_string();
    let mut write_clone = write.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if write_clone.send(Message::Ping(vec![])).await.is_err() {
                break;
            }
        }
    });
    
    // Process incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(trade) = parse_trade_event(&event, wallet) {
                        if let Err(e) = tx.send(trade).await {
                            tracing::error!("Failed to send trade to channel: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Pong(_)) => {
                // Connection alive
            }
            Ok(Message::Close(_)) => {
                tracing::warn!("WebSocket closed for {}", wallet);
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn parse_trade_event(event: &serde_json::Value, wallet: &str) -> Option<Trade> {
    let event_type = event["type"].as_str()?;
    
    if event_type != "trade" {
        return None;
    }
    
    let data = &event["data"];
    
    Some(Trade {
        wallet: wallet.to_string(),
        event_id: data["event_id"].as_str()?.to_string(),
        market_id: data["market_id"].as_str()?.to_string(),
        side: match data["side"].as_str()? {
            "BUY" => TradeSide::BUY,
            "SELL" => TradeSide::SELL,
            _ => return None,
        },
        shares: data["shares"].as_f64()?,
        price: data["price"].as_f64()?,
        timestamp: data["timestamp"].as_i64()?,
        tx_hash: data["tx_hash"].as_str().map(|s| s.to_string()),
    })
}