mod types;
mod config;
mod api;
mod watcher;
mod sizing;
mod risk;
mod executor;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("🚀 Polymarket Copy Trading Bot Starting...");
    
    // Load configuration
    let config = config::load_config()?;
    config::validate_config(&config)?;
    
    tracing::info!("✅ Configuration loaded");
    tracing::info!("   Tracking {} wallets", config.wallets_to_track.len());
    tracing::info!("   Sizing mode: {:?}", config.sizing_mode);
    tracing::info!("   Your wallet: {}", &config.your_wallet[..10]);
    
    // Initialize components
    let api = api::PolymarketApi::new(config.polymarket_api.clone());
    let watcher = watcher::WalletWatcher::new(
        config.ws_url.clone(),
        config.wallets_to_track.clone(),
    );
    let sizer = sizing::PositionSizer::new(config.clone());
    let risk = risk::RiskManager::new(config.clone());
    let executor = executor::TradeExecutor::new(api.clone(), config.clone());
    
    tracing::info!("✅ Components initialized");
    
    // Start watching wallets
    let trade_rx = watcher.start().await?;
    tracing::info!("✅ WebSocket watchers started");
    
    // Reset daily stats at midnight
    let risk_clone = risk.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let now = chrono::Utc::now();
            if now.hour() == 0 && now.minute() < 1 {
                risk_clone.reset_daily_stats();
            }
        }
    });
    
    // Main trading loop
    tracing::info!("🎯 Bot is now live and monitoring trades...");
    
    while let Ok(whale_trade) = trade_rx.recv().await {
        tracing::info!("📊 Detected trade from {}: {} {:.2} shares @ ${:.4}",
            &whale_trade.wallet[..10],
            match whale_trade.side {
                types::TradeSide::BUY => "BUY",
                types::TradeSide::SELL => "SELL",
            },
            whale_trade.shares,
            whale_trade.price
        );
        
        // Verify whale
        if !risk.is_whale_verified(&whale_trade.wallet) {
            tracing::warn!("⚠️  Unverified wallet, skipping");
            continue;
        }
        
        // Get market info
        let market = match api.get_market(&whale_trade.market_id).await {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to fetch market: {}", e);
                risk.record_error(&format!("Market fetch failed: {}", e));
                continue;
            }
        };
        
        tracing::info!("   Market: {}", market.question);
        tracing::info!("   Liquidity: ${:.2}", market.liquidity);
        
        // Get balances
        let your_balance = match api.get_balance(&config.your_wallet).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to fetch your balance: {}", e);
                risk.record_error(&format!("Balance fetch failed: {}", e));
                continue;
            }
        };
        
        let whale_balance = match api.get_balance(&whale_trade.wallet).await {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("Failed to fetch whale balance: {}", e);
                1000000.0 // Default to large number if we can't fetch
            }
        };
        
        // Calculate position size
        let size_usd = match sizer.calculate_size(&whale_trade, your_balance, whale_balance).await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to calculate size: {}", e);
                risk.record_error(&format!("Sizing failed: {}", e));
                continue;
            }
        };
        
        let shares = sizer.shares_from_usd(size_usd, whale_trade.price);
        
        tracing::info!("   Your size: ${:.2} ({:.2} shares)", size_usd, shares);
        
        // Risk checks
        if let Err(e) = risk.check_can_trade(&whale_trade, &market, size_usd) {
            tracing::error!("❌ Risk check failed: {}", e);
            continue;
        }
        
        tracing::info!("✅ Risk checks passed");
        
        // Execute trade
        tracing::info!("🔄 Executing mirror trade...");
        
        match executor.execute_trade(&whale_trade, shares).await {
            Ok(resp) => {
                tracing::info!("✅ Trade executed successfully!");
                tracing::info!("   Order ID: {}", resp.order_id);
                tracing::info!("   Filled: {:.2} shares @ ${:.4}", resp.filled_shares, resp.avg_fill_price);
                tracing::info!("   Total: ${:.2}", resp.filled_shares * resp.avg_fill_price);
                
                risk.record_trade(&whale_trade, size_usd);
            }
            Err(e) => {
                tracing::error!("❌ Trade execution failed: {}", e);
                risk.record_error(&format!("Execution failed: {}", e));
            }
        }
        
        // Show circuit breaker status
        let cb_state = risk.get_state();
        tracing::info!("📈 Daily stats: {} trades, ${:.2} volume", 
            cb_state.total_trades_today, 
            cb_state.total_volume_today
        );
        
        if cb_state.is_tripped {
            tracing::error!("⚠️  CIRCUIT BREAKER TRIPPED - Bot paused!");
        }
        
        tracing::info!("---");
    }
    
    tracing::info!("Bot stopped");
    Ok(())
}