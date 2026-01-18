# 🔁 Polymarket Copy Trading Bot - Full Implementation

**Complete Rust implementation** with all missing features from the public GitHub repo.

## ✅ What's Included

This implementation includes ALL the missing code:

### Core Modules
- ✅ `src/main.rs` - Main orchestrator with WebSocket integration
- ✅ `src/types.rs` - Complete type definitions
- ✅ `src/config.rs` - Configuration loading and validation
- ✅ `src/api.rs` - Full Polymarket API client
- ✅ `src/watcher.rs` - Real-time WebSocket wallet monitoring
- ✅ `src/sizing.rs` - Position sizing (Fixed/Proportional/Tier-based)
- ✅ `src/risk.rs` - Circuit breaker and risk management
- ✅ `src/executor.rs` - Trade execution with retry logic

### Binaries
- ✅ `src/bin/mempool_monitor.rs` - Mempool monitoring for frontrunning

### Features
- ✅ **WebSocket real-time monitoring** (not HTTP polling)
- ✅ **Circuit breaker** with consecutive error tracking
- ✅ **Multi-wallet tracking** with concurrent subscriptions
- ✅ **Intelligent position sizing** (3 modes)
- ✅ **Risk management** (exposure limits, liquidity checks)
- ✅ **Retry logic** with exponential backoff
- ✅ **Order type optimization** (FAK for buys, GTD for sells)
- ✅ **Mempool monitoring** for same-block execution
- ✅ **Automatic reconnection** on WebSocket disconnect

---

## 🚀 Installation

### Prerequisites

1. **Install Rust**: https://rustup.rs/
2. **Get API Keys**:
   - Alchemy: https://www.alchemy.com/ (for RPC)
   - Or Infura: https://infura.io/
3. **Polymarket Account** with funded wallet

### Setup

```bash
# 1. Create project directory
mkdir polymarket-copy-bot
cd polymarket-copy-bot

# 2. Copy all the artifact files into their correct locations:
#    - Cargo.toml in root
#    - All .rs files in src/
#    - mempool_monitor.rs in src/bin/

# 3. Copy .env.example to .env
cp .env.example .env

# 4. Edit .env with your settings
nano .env
```

### Configure .env

```env
# REQUIRED: Wallets you want to copy
WALLETS_TO_TRACK=0xWHALE_ADDRESS_1,0xWHALE_ADDRESS_2

# REQUIRED: Your wallet
YOUR_WALLET=0xYourAddress

# REQUIRED: Your private key (KEEP SECRET!)
PRIVATE_KEY=0xYourPrivateKey

# REQUIRED: RPC endpoint (Polygon for Polymarket)
RPC_URL=wss://polygon-mainnet.g.alchemy.com/v2/YOUR_ALCHEMY_KEY

# Sizing (choose one mode)
SIZING_MODE=fixed           # or "proportional" or "tierbased"
FIXED_STAKE=25.0           # USD per trade
```

---

## 🎯 Running the Bot

### Standard Mode (WebSocket)

```bash
# Build in release mode (optimized)
cargo build --release

# Run the bot
cargo run --release --bin polymarket-bot
```

### Mempool Mode (Advanced)

⚠️ **Warning**: Mempool monitoring is more aggressive and risky

```bash
# Run mempool monitor
cargo run --release --bin mempool-monitor
```

---

## 📊 How It Works

```
┌─────────────────────────────────────────────┐
│  Whale Wallet (via WebSocket)              │
│  Makes trade: BUY 100 shares @ $0.65       │
└──────────────────┬──────────────────────────┘
                   │ <150ms
                   ▼
┌─────────────────────────────────────────────┐
│  Bot Detects Trade                         │
│  - Validates whale                         │
│  - Fetches market data                     │
│  - Checks liquidity                        │
└──────────────────┬──────────────────────────┘
                   ▼
┌─────────────────────────────────────────────┐
│  Position Sizing                           │
│  Fixed: $25                                │
│  or Proportional: (your_bal/whale_bal)*amt│
│  or Tier-based: multiplier based on size  │
└──────────────────┬──────────────────────────┘
                   ▼
┌─────────────────────────────────────────────┐
│  Risk Checks                               │
│  ✓ Circuit breaker OK?                     │
│  ✓ Daily volume < limit?                  │
│  ✓ Event exposure < limit?                │
│  ✓ Sufficient liquidity?                  │
└──────────────────┬──────────────────────────┘
                   ▼
┌─────────────────────────────────────────────┐
│  Execute Trade                             │
│  - FAK order (BUY) or GTD order (SELL)    │
│  - Retry up to 4 times                    │
│  - Record trade in stats                  │
└─────────────────────────────────────────────┘
```

---

## ⚙️ Configuration Options

### Sizing Modes

**1. Fixed Mode** (Simplest)
```env
SIZING_MODE=fixed
FIXED_STAKE=25.0
```
Every trade = $25 USD

**2. Proportional Mode**
```env
SIZING_MODE=proportional
PROPORTIONAL_RATIO=0.02
```
Your stake = (Your Balance / Whale Balance) × Whale Stake

**3. Tier-Based Mode** (Smart)
```env
SIZING_MODE=tierbased
PROPORTIONAL_RATIO=0.02
```
Multipliers:
- <$50 whale trade: 0.5x weight
- $50-200: 1.0x weight
- $200-500: 1.5x weight
- >$500: 2.0x weight

### Risk Limits

```env
# Max exposure per event
MAX_EXPOSURE_PER_EVENT=500.0

# Max total daily volume
MAX_DAILY_VOLUME=2000.0

# Minimum market liquidity required
MIN_LIQUIDITY=1000.0

# Circuit breaker: trip after N consecutive errors
CB_CONSECUTIVE_TRIGGER=3

# Minimum orderbook depth
CB_MIN_DEPTH_USD=100.0
```

---

## 🛡️ Safety Features

### 1. Circuit Breaker
Automatically stops trading after:
- 3 consecutive errors (configurable)
- Protects against API issues, network problems

### 2. Exposure Limits
- Max $ per event
- Max $ per day
- Prevents overexposure

### 3. Liquidity Checks
- Requires minimum liquidity
- Checks orderbook depth
- Prevents slippage on thin markets

### 4. Whale Verification
- Only copies whitelisted wallets
- Ignores unknown addresses

### 5. Retry Logic
- 4 attempts with exponential backoff
- Handles temporary API failures

---

## 📈 Monitoring

The bot logs everything:

```
🚀 Polymarket Copy Trading Bot Starting...
✅ Configuration loaded
   Tracking 2 wallets
   Sizing mode: Fixed
   Your wallet: 0x12345678...
✅ Components initialized
✅ WebSocket watchers started
🎯 Bot is now live and monitoring trades...

📊 Detected trade from 0x1234567...: BUY 150.00 shares @ $0.6500
   Market: Will Bitcoin hit $100k in 2024?
   Liquidity: $45000.00
   Your size: $25.00 (38.46 shares)
✅ Risk checks passed
🔄 Executing mirror trade...
✅ Trade executed successfully!
   Order ID: abc-123
   Filled: 38.46 shares @ $0.6502
   Total: $25.01
📈 Daily stats: 1 trades, $25.01 volume
---
```

---

## ⚠️ Important Warnings

### Security
- **NEVER commit your `.env` file**
- **NEVER share your PRIVATE_KEY**
- Store private key securely (consider hardware wallet)

### Risks
- ✗ Copy trading doesn't guarantee profits
- ✗ Whales can manipulate or have insider info
- ✗ You'll always be slightly behind the whale
- ✗ Fees and slippage reduce profits
- ✗ Polymarket may block bot activity

### Legal
- ✗ Check if bots are allowed in your jurisdiction
- ✗ You're responsible for tax reporting
- ✗ No warranty - use at your own risk

---

## 🔧 Troubleshooting

### "WebSocket connection failed"
- Check your `WS_URL`
- Verify internet connection
- Try different WebSocket endpoint

### "Failed to place order"
- Check PRIVATE_KEY is correct
- Verify wallet has sufficient balance
- Check Polymarket API status

### "Circuit breaker tripped"
- Check logs for repeated errors
- Fix underlying issue
- Reset with: modify `src/main.rs` to add reset endpoint

### "RPC_URL not set"
- Get API key from Alchemy or Infura
- Set `RPC_URL` in `.env`

---

## 🎓 Testing

```bash
# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run --release --bin polymarket-bot

# Test specific module
cargo test --lib sizing
```

---

## 💡 Tips for Success

1. **Start Small**: Begin with MIN_STAKE and small limits
2. **Monitor First**: Watch bot for a day before increasing stakes
3. **Choose Good Whales**: Track whales with proven track records
4. **Use Tier-Based**: Gives more weight to whale's larger trades
5. **Set Strict Limits**: Better to miss trades than lose money
6. **Monitor Liquidity**: High liquidity = better execution
7. **Track Performance**: Log all trades, calculate P&L

---

## 📝 Improvements You Can Add

- [ ] Add PostgreSQL for trade history
- [ ] Add Telegram notifications
- [ ] Add profit/loss tracking dashboard
- [ ] Add whale performance scoring
- [ ] Add auto-exit on profit target
- [ ] Add stop-loss functionality
- [ ] Add multi-chain support

---

## 🆘 Support

If you have issues:
1. Check logs carefully
2. Verify all `.env` variables
3. Test with small amounts first
4. Read error messages - they're usually clear

---

## 📜 License

MIT License - Use at your own risk

---

## ⚡ Performance

Expected performance:
- **Latency**: <150ms from whale trade to your execution
- **Memory**: 50-200 MB RAM
- **CPU**: Low (async/await is efficient)
- **Network**: ~10 KB/s (WebSocket traffic)

---

**Remember**: This bot gives you the CODE, but success depends on:
- Choosing the right whales to copy
- Setting appropriate risk limits
- Managing your bankroll
- Understanding market dynamics

Trade responsibly! 🚀