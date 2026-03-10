# Hyperliquid CLI (`hl`)

You have access to the `hl` command-line tool for interacting with the Hyperliquid decentralized exchange.
It is installed on PATH via `cargo install`.

## Environment

- `HL_PRIVATE_KEY` must be set for write operations (placing orders, cancelling, etc.)
- `HL_ADDRESS` can be set for read-only operations if no private key is available
- `HL_NETWORK` can be set to `mainnet` or `testnet` (default: mainnet)
- Always use `--network testnet` when testing
- Environment variables can be set in `~/.hl.env` (global) or `.env` in the working directory

## IMPORTANT SAFETY RULES

1. **ALWAYS confirm with the user before executing ANY write operation** (placing orders, cancelling orders, modifying orders, setting leverage, transferring funds, withdrawing)
2. **ALWAYS use `--json` flag** when you need to parse the output programmatically
3. **ALWAYS use `--network testnet` for testing** unless the user explicitly asks for mainnet
4. **NEVER place orders without the user's explicit confirmation** of: coin, side (buy/sell), size, price, and network

## Read Commands (no confirmation needed)

```bash
# Account state (margin, equity)
hl state --json

# Open positions
hl positions --json

# Token balances
hl balance --json

# Open orders
hl orders --json

# Check specific order status
hl order-status --oid 123456 --json

# Recent fills/trades for your account
hl fills --json

# Historical orders
hl historical-orders --json

# L2 order book
hl book ETH --json

# All mid prices
hl mids --json

# Exchange metadata (asset list with indices)
hl meta --json

# Spot metadata
hl spot-meta --json

# Funding rate history
hl funding ETH --start 1700000000000 --json

# User funding history
hl user-funding --start 1700000000000 --json

# Candle data
hl candles ETH --interval 1h --start 1700000000000 --end 1700100000000 --json

# Recent trades for a coin
hl trades ETH --json
```

## Write Commands (ALWAYS confirm first)

```bash
# Place a limit buy order (GTC)
hl order place ETH buy --size 0.1 --price 3000 --tif gtc --json

# Place a limit sell order
hl order place BTC sell --size 0.01 --price 100000 --tif gtc --json

# Place a reduce-only order
hl order place ETH sell --size 0.1 --price 4000 --reduce-only --json

# Place an IOC order
hl order place ETH buy --size 0.5 --price 3000 --tif ioc --json

# Place a stop-loss trigger order
hl order place ETH sell --size 0.1 --price 2800 --trigger-price 2850 --trigger-type sl --json

# Place a take-profit trigger order
hl order place ETH sell --size 0.1 --price 3500 --trigger-price 3400 --trigger-type tp --json

# Cancel an order
hl order cancel ETH --oid 123456 --json

# Cancel by client order ID
hl order cancel-by-cloid ETH --cloid my-order-1 --json

# Modify an order
hl order modify --oid 123456 ETH buy --size 0.2 --price 3100 --json

# Set leverage
hl leverage set ETH 10 --mode cross --json
hl leverage set BTC 5 --mode isolated --json

# Transfer USDC
hl transfer --to 0x1234...abcd --amount 100 --json

# Withdraw USDC to L1
hl withdraw --to 0x1234...abcd --amount 100 --json
```

## HIP-3 Builder-Deployed Perps

HIP-3 dexes are permissionless perp markets deployed by builders (e.g. `xyz` for stocks, `km` for indices/forex, `flx` for various). Use `--dex` flag or `dex:coin` syntax.

```bash
# List available HIP-3 dexes
hl dexes --json

# Read commands (two equivalent syntaxes)
hl --dex xyz meta --json              # Asset list for xyz dex
hl --dex xyz mids --json              # Mid prices for xyz dex
hl book xyz:TSLA --json               # Order book (dex:coin syntax)
hl --dex xyz book TSLA --json         # Order book (--dex flag)
hl trades xyz:TSLA --json             # Recent trades
hl funding xyz:TSLA --start 1700000000000 --json

# Account state on a specific dex
hl --dex xyz state --json
hl --dex xyz positions --json

# Write commands on HIP-3 dexes (ALWAYS confirm first)
hl --dex xyz order place TSLA buy --size 1 --price 400 --json
hl --dex xyz order cancel TSLA --oid 123456 --json
hl --dex xyz leverage set TSLA 5 --mode cross --json
```

## Network Selection

```bash
# Use testnet
hl --network testnet positions --json

# Use mainnet explicitly
hl --network mainnet state --json
```

## Output Formats

- **Default**: Human-readable tables (good for display to user)
- **`--json`**: Machine-readable JSON (use this when parsing output)

## Setup: `hl init`

When ARGUMENTS is "init", use AskUserQuestion to collect config values interactively, then run `hl init` with flags. Do NOT run `hl init` without flags (stdin prompts don't work in Claude Code).

`hl init` does two things: writes `~/.hl.env` AND installs the Claude Code skill to `~/.claude/commands/hl.md`.

Prompt the user with these questions using AskUserQuestion:
1. **Wallet address** (0x...) — for read-only operations. Options: "Skip for now" or "Enter address" (user types via Other).
2. **API Wallet private key** — for write operations. Options: "Skip for now" or "Enter private key" (user types via Other). Mention: create one at Hyperliquid UI → Settings → API Wallets.
3. **Default network** — Options: "mainnet (Recommended)" or "testnet".

Then run:
```bash
hl init --address <ADDR> --private-key <KEY> --network <NET> [--force]
```
Omit `--address` or `--private-key` if the user skipped them. Add `--force` if `~/.hl.env` already exists and user wants to overwrite.

## Workflow Examples

### Check account status
```bash
hl state --json
hl positions --json
hl orders --json
```

### Place and monitor an order
```bash
# 1. Check current price
hl book ETH --json
# 2. Place order (after user confirmation)
hl order place ETH buy --size 0.1 --price 3000 --json
# 3. Check order status
hl orders --json
```

### Risk management
```bash
# Set leverage before trading
hl leverage set ETH 5 --mode cross --json
# Place main order
hl order place ETH buy --size 1.0 --price 3000 --json
# Place stop-loss
hl order place ETH sell --size 1.0 --price 2800 --trigger-price 2850 --trigger-type sl --json
```
