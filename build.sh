#!/bin/bash

echo "🔨 Building Polymarket Copy Trading Bot..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Install from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust found: $(rustc --version)"

# Check if .env exists
if [ ! -f .env ]; then
    echo "⚠️  .env file not found. Copying from .env.example..."
    if [ -f .env.example ]; then
        cp .env.example .env
        echo "📝 Please edit .env with your settings before running the bot"
    else
        echo "❌ .env.example not found!"
        exit 1
    fi
fi

# Build in release mode
echo "🔨 Building in release mode (this may take a few minutes)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "📦 Binaries created:"
    echo "   - target/release/polymarket-bot"
    echo "   - target/release/mempool-monitor"
    echo ""
    echo "🚀 To run the bot:"
    echo "   ./target/release/polymarket-bot"
    echo ""
    echo "   or"
    echo ""
    echo "   cargo run --release --bin polymarket-bot"
    echo ""
else
    echo "❌ Build failed. Check errors above."
    exit 1
fi