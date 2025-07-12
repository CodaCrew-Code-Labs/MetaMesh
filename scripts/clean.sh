#!/bin/bash
echo "🧹 Cleaning MetaMesh build artifacts..."
cargo clean
rm -rf dist/
rm -rf coverage/
echo "✅ Cleanup complete!"