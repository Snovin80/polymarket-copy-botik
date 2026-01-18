# 📁 Project Structure - Complete Overview

## Полная структура проекта

```
polymarket-copy-bot/
│
├── Cargo.toml                          # Зависимости и настройки Rust
├── Cargo.lock                          # Зафиксированные версии (автогенерируется)
│
├── .env                                # Ваши настройки (НЕ коммитить!)
├── .env.example                        # Пример настроек
├── .gitignore                          # Git исключения
│
├── build.sh                            # Скрипт сборки (опционально)
│
├── README_FULL.md                      # Полная документация
├── DEPLOYMENT.md                       # Гайд по развертыванию
├── QUICKSTART.md                       # Быстрый старт
├── PROJECT_STRUCTURE.md                # Этот файл
│
├── src/                                # Исходный код
│   ├── main.rs                         # Точка входа, главный оркестратор
│   ├── types.rs                        # Типы данных (Trade, Market, Config, etc.)
│   ├── config.rs                       # Загрузка конфигурации из .env
│   ├── api.rs                          # Polymarket API клиент (HTTP)
│   ├── watcher.rs                      # WebSocket мониторинг кошельков
│   ├── sizing.rs                       # Расчет размера позиций
│   ├── risk.rs                         # Circuit breaker и риск-менеджмент
│   ├── executor.rs                     # Исполнение сделок с retry логикой
│   │
│   └── bin/                            # Дополнительные бинарники
│       └── mempool_monitor.rs          # Мониторинг mempool (advanced)
│
├── tests/                              # Тесты (опционально)
│   └── integration_test.rs             # Интеграционные тесты
│
└── target/                             # Скомпилированные бинарники (автогенерируется)
    └── release/
        ├── polymarket-bot              # Основной бот
        └── mempool-monitor             # Mempool монитор
```

---

## 📦 Описание файлов

### Конфигурация

#### `Cargo.toml`
```toml
[package]
name = "polymarket-copy-bot"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "polymarket-bot"
path = "src/main.rs"

[[bin]]
name = "mempool-monitor"
path = "src/bin/mempool_monitor.rs"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
# ... остальные зависимости
```

**Назначение**: Определяет проект, зависимости, бинарники

---

#### `.env` (создайте из `.env.example`)
```env
WALLETS_TO_TRACK=0x...
YOUR_WALLET=0x...
PRIVATE_KEY=...
RPC_URL=wss://...
SIZING_MODE=fixed
FIXED_STAKE=25.0
# ... остальные настройки
```

**Назначение**: Ваша конфигурация (приватные ключи, адреса)

**⚠️ ВАЖНО**: НЕ коммитьте этот файл в Git!

---

### Исходный код

#### `src/main.rs` (1024+ строки)
```rust
mod types;
mod config;
mod api;
mod watcher;
mod sizing;
mod risk;
mod executor;

#[tokio::main]
async fn main() -> Result<()> {
    // Инициализация логирования
    // Загрузка конфига
    // Создание компонентов
    // Запуск WebSocket watchers
    // Главный цикл обработки сделок
}
```

**Назначение**: 
- Точка входа программы
- Оркестрирует все компоненты
- Главный цикл копирования сделок

**Основной flow**:
1. Загрузка конфигурации
2. Инициализация API, WebSocket, Risk Manager
3. Подписка на кошельки китов
4. При детекте сделки:
   - Проверка risk limits
   - Расчет размера позиции
   - Исполнение зеркальной сделки

---

#### `src/types.rs` (128+ строки)
```rust
pub struct Trade { ... }
pub struct Market { ... }
pub struct Position { ... }
pub struct Config { ... }
pub enum TradeSide { BUY, SELL }
pub enum OrderType { MARKET, LIMIT, FAK, GTD }
// ...
```

**Назначение**: Определение всех типов данных

**Основные типы**:
- `Trade` - информация о сделке
- `Market` - данные о рынке
- `Config` - конфигурация бота
- `TradeSide` - направление (BUY/SELL)
- `OrderType` - тип ордера
- `CircuitBreakerState` - состояние защиты

---

#### `src/config.rs` (96+ строки)
```rust
pub fn load_config() -> Result<Config> {
    dotenv::dotenv().ok();
    // Загрузка из env variables
}

pub fn validate_config(config: &Config) -> Result<()> {
    // Проверка корректности
}
```

**Назначение**: Загрузка и валидация конфигурации

**Функции**:
- Чтение переменных окружения из `.env`
- Парсинг значений в правильные типы
- Валидация (проверка формата адресов, лимитов)
- Предоставление defaults

---

#### `src/api.rs` (256+ строки)
```rust
pub struct PolymarketApi {
    client: Client,
    base_url: String,
}

impl PolymarketApi {
    pub async fn get_market(&self, market_id: &str) -> Result<Market>
    pub async fn get_trades(&self, wallet: &str, since: i64) -> Result<Vec<Trade>>
    pub async fn get_orderbook(&self, market_id: &str) -> Result<...>
    pub async fn place_order(&self, req: OrderRequest, api_key: &str) -> Result<OrderResponse>
    pub async fn get_balance(&self, wallet: &str) -> Result<f64>
}
```

**Назначение**: HTTP клиент для Polymarket API

**Основные методы**:
- `get_market()` - данные о рынке (ликвидность, цены)
- `get_trades()` - история сделок кошелька
- `get_orderbook()` - стакан ордеров
- `place_order()` - размещение ордера
- `get_balance()` - баланс кошелька

---

#### `src/watcher.rs` (192+ строки)
```rust
pub struct WalletWatcher {
    ws_url: String,
    wallets: Vec<String>,
}

impl WalletWatcher {
    pub async fn start(&self) -> Result<Receiver<Trade>>
}

async fn watch_wallet(ws_url: String, wallet: String, tx: Sender<Trade>) -> Result<()>
```

**Назначение**: Real-time мониторинг кошельков через WebSocket

**Как работает**:
1. Подключение к WebSocket Polymarket
2. Подписка на trades конкретного wallet
3. Парсинг входящих сообщений
4. Отправка Trade events в channel
5. Auto-reconnect при разрыве

**Преимущества**:
- ✅ Реальное время (<150ms latency)
- ✅ Не нужно polling
- ✅ Эффективное использование ресурсов

---

#### `src/sizing.rs` (128+ строки)
```rust
pub struct PositionSizer {
    config: Config,
}

impl PositionSizer {
    pub async fn calculate_size(&self, whale_trade: &Trade, your_balance: f64, whale_balance: f64) -> Result<f64>
    fn get_tier_multiplier(&self, trade_size_usd: f64) -> f64
    pub fn shares_from_usd(&self, usd_amount: f64, price: f64) -> f64
}
```

**Назначение**: Расчет размера позиций

**3 режима sizing**:

1. **Fixed** - фиксированная сумма каждый раз
   ```rust
   size = FIXED_STAKE // например $25
   ```

2. **Proportional** - пропорционально киту
   ```rust
   ratio = your_balance / whale_balance
   size = whale_trade_size * ratio
   ```

3. **Tier-Based** - умные множители
   ```rust
   multiplier = match whale_trade_size {
       0..50 => 0.5,
       50..200 => 1.0,
       200..500 => 1.5,
       500+ => 2.0,
   }
   size = whale_shares * multiplier * proportional_ratio
   ```

---

#### `src/risk.rs` (256+ строки)
```rust
pub struct RiskManager {
    config: Config,
    state: Arc<Mutex<CircuitBreakerState>>,
    event_exposure: Arc<Mutex<HashMap<String, f64>>>,
}

impl RiskManager {
    pub fn check_can_trade(&self, trade: &Trade, market: &Market, size_usd: f64) -> Result<()>
    pub fn record_trade(&self, trade: &Trade, size_usd: f64)
    pub fn record_error(&self, error: &str)
    pub fn reset_circuit_breaker(&self)
}
```

**Назначение**: Управление рисками и circuit breaker

**Проверки перед каждой сделкой**:
1. ✅ Circuit breaker не сработал?
2. ✅ Дневной лимит не превышен?
3. ✅ Экспозиция по event не превышена?
4. ✅ Достаточная ликвидность?
5. ✅ Достаточная глубина orderbook?
6. ✅ Кит верифицирован?

**Circuit Breaker**:
- Срабатывает после N consecutive errors
- Блокирует все сделки
- Требует ручного сброса или автосброс через время

---

#### `src/executor.rs` (192+ строки)
```rust
pub struct TradeExecutor {
    api: PolymarketApi,
    config: Config,
}

impl TradeExecutor {
    pub async fn execute_trade(&self, trade: &Trade, shares: f64) -> Result<OrderResponse>
    async fn execute_with_retry(&self, order: OrderRequest) -> Result<OrderResponse>
    pub async fn close_position(&self, market_id: &str, shares: f64, side: TradeSide) -> Result<OrderResponse>
}
```

**Назначение**: Исполнение сделок с retry логикой

**Оптимизация типов ордеров**:
- BUY → FAK (Fill-And-Kill) - быстрое исполнение
- SELL → GTD (Good-Till-Date) - ждет лучшей цены

**Retry logic**:
- До 4 попыток
- Exponential backoff (500ms, 1000ms, 1500ms, 2000ms)
- Умная обработка ошибок

---

#### `src/bin/mempool_monitor.rs` (192+ строки)
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Подключение к Ethereum/Polygon RPC
    // Подписка на pending transactions
    // Фильтрация транзакций от tracked wallets
    // Парсинг trade data
    // Опциональное исполнение mirror trade
}
```

**Назначение**: Advanced - мониторинг mempool

**Как работает**:
1. Подключение к Polygon RPC (Alchemy/Infura)
2. Подписка на `pending_transactions`
3. Детект транзакций от отслеживаемых кошельков
4. Проверка что это Polymarket contract
5. Парсинг trade details из tx.input
6. **Исполнение mirror trade ДО того как оригинал включен в блок**

**Преимущества**:
- ✅ Same-block execution
- ✅ Опережаете всех HTTP-based ботов

**Риски**:
- ⚠️ Транзакция кита может не пройти (revert)
- ⚠️ Нужен платный RPC с mempool support
- ⚠️ Более сложная настройка

---

## 🔄 Flow исполнения

### Стандартный режим (WebSocket)

```
1. Bot starts
   ↓
2. Load config from .env
   ↓
3. Initialize components:
   - API client
   - WebSocket watcher
   - Position sizer
   - Risk manager
   - Trade executor
   ↓
4. Subscribe to whale wallets via WebSocket
   ↓
5. Wait for trade events...
   ↓
6. Trade detected!
   ├─→ Verify whale
   ├─→ Fetch market data
   ├─→ Get balances
   ├─→ Calculate position size
   ├─→ Risk checks
   ├─→ Execute mirror trade
   └─→ Record trade stats
   ↓
7. Repeat from step 5
```

### Mempool режим (Advanced)

```
1. Bot starts (mempool-monitor)
   ↓
2. Connect to Polygon RPC
   ↓
3. Subscribe to pending_txs
   ↓
4. For each pending tx:
   ├─→ Check if from tracked wallet?
   ├─→ Check if Polymarket contract?
   ├─→ Parse trade details
   └─→ Execute mirror trade (before original is mined!)
   ↓
5. Repeat
```

---

## 🧪 Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_test

# Specific module
cargo test --lib sizing

# With output
cargo test -- --nocapture

# Run mempool monitor in test mode
RUST_LOG=debug cargo run --bin mempool-monitor
```

---

## 📊 Logging

Bot использует `tracing` для логирования:

```rust
tracing::info!("Info message");
tracing::warn!("Warning message");
tracing::error!("Error message");
tracing::debug!("Debug message"); // только с RUST_LOG=debug
```

**Управление логами**:
```bash
# Default (info level)
cargo run --release

# Debug mode
RUST_LOG=debug cargo run --release

# Specific module
RUST_LOG=polymarket_copy_bot::watcher=debug cargo run --release

# Только ошибки
RUST_LOG=error cargo run --release
```

---

## 🔐 Security Considerations

### Что НЕ коммитить в Git:

```gitignore
# .gitignore
.env
target/
Cargo.lock  # опционально
*.log
*.csv
```

### Права доступа на .env:

```bash
chmod 600 .env  # только владелец может читать/писать
```

### Где хранить приватный ключ:

❌ **НЕ хранить**:
- В Git репозитории
- В облачных хранилищах без шифрования
- В текстовых файлах без защиты

✅ **Рекомендуется**:
- В `.env` с правами 600
- В зашифрованном хранилище (1Password, Bitwarden)
- В hardware wallet (для production)
- В отдельном кошельке только для бота

---

## 📈 Performance

**Metrics**:
- Latency: <150ms (WebSocket) или <50ms (Mempool)
- Memory: 50-200 MB RAM
- CPU: <5% на современных процессорах
- Network: ~10 KB/s WebSocket traffic

**Оптимизация**:
- Используйте `--release` build (10-100x faster)
- Запускайте на VPS близко к Polygon RPC
- Используйте платный RPC с высокими лимитами

---

## 🛠️ Maintenance

### Обновление зависимостей:

```bash
cargo update
cargo build --release
```

### Проверка устаревших зависимостей:

```bash
cargo install cargo-outdated
cargo outdated
```

### Проверка безопасности:

```bash
cargo install cargo-audit
cargo audit
```

---

## 📝 Changelog

### Version 0.1.0 (Current)
- ✅ WebSocket real-time monitoring
- ✅ Circuit breaker
- ✅ 3 sizing modes
- ✅ Risk management
- ✅ Retry logic
- ✅ Mempool monitor

### Future (Planned)
- [ ] PostgreSQL trade history
- [ ] Telegram notifications
- [ ] Web dashboard
- [ ] Auto profit-taking
- [ ] Stop-loss
- [ ] Multi-chain support

---

Этот файл дает полное представление о структуре проекта. Используйте его как reference при разработке или дебаге!