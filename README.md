# hl — Hyperliquid CLI for AI Agents

A Rust CLI tool for interacting with the [Hyperliquid](https://hyperliquid.xyz) decentralized exchange, built on [hl-rs](https://github.com/kinetiq-research/hl-rs). Designed as a tool for AI agents (e.g., [Claude Code](https://claude.com/claude-code), OpenClaw) to read market data, manage positions, and place orders on Hyperliquid.

## Install

```bash
cargo install --git https://github.com/kinetiq-research/hl-cli.git
```

This installs `hl` to `~/.cargo/bin/` (already on PATH).

## Setup

```bash
hl init
```

That's it. This configures `~/.hl.env` and installs the Claude Code skill to `~/.claude/commands/hl.md` — then invoke with `/hl` in any Claude Code session.

```bash
hl init --no-skill    # skip skill installation
hl init --force       # overwrite existing config and skill
```

### Manual credential setup

Alternatively, create `~/.hl.env` directly:

```bash
# Auth (write operations) — use an API Wallet private key
# Create one at: Hyperliquid UI → Settings → API Wallets
HL_PRIVATE_KEY=0x...

# Auth (read-only) — your wallet address
HL_ADDRESS=0x...

# Optional: default network (mainnet or testnet)
HL_NETWORK=mainnet
```

## Usage

### Global Flags

| Flag | Description |
|------|-------------|
| `--network mainnet\|testnet` | Target network (env: `HL_NETWORK`, default: mainnet) |
| `--json` | Machine-readable JSON output |
| `--dex <name>` | HIP-3 builder-deployed perp dex (env: `HL_DEX`, e.g. `xyz`, `km`, `flx`) |

### Read Commands

```bash
# Account
hl state                          # Margin, equity, withdrawable
hl positions                      # Open positions with PnL
hl balance                        # Token balances
hl orders                         # Open orders
hl order-status --oid 123456      # Single order status
hl fills                          # Recent fills
hl historical-orders              # Past orders

# Market data
hl meta                           # Perp asset list (name → index mapping)
hl spot-meta                      # Spot token/pair metadata
hl mids                           # All mid prices (spot names resolved)
hl book ETH                       # L2 order book
hl trades ETH                     # Recent trades

# Funding & candles
hl funding ETH --start 1700000000000
hl user-funding --start 1700000000000
hl candles ETH --interval 1h --start 1700000000000 --end 1700100000000
```

### Write Commands

```bash
# Orders
hl order place ETH buy --size 0.1 --price 3000                        # Limit GTC
hl order place ETH buy --size 0.1 --price 3000 --tif ioc              # IOC
hl order place ETH sell --size 0.1 --price 4000 --reduce-only         # Reduce-only
hl order place ETH sell --size 0.1 --price 2800 \
  --trigger-price 2850 --trigger-type sl                               # Stop-loss
hl order place ETH sell --size 0.1 --price 3500 \
  --trigger-price 3400 --trigger-type tp                               # Take-profit

hl order cancel ETH --oid 123456                                       # Cancel by ID
hl order cancel-by-cloid ETH --cloid my-order-1                       # Cancel by client ID
hl order modify --oid 123456 ETH buy --size 0.2 --price 3100          # Modify

# Leverage
hl leverage set ETH 10 --mode cross
hl leverage set BTC 5 --mode isolated

# Transfers
hl transfer --to 0x1234...abcd --amount 100       # USDC transfer
hl withdraw --to 0x1234...abcd --amount 100        # Withdraw to L1
```

### HIP-3 Builder-Deployed Perps

HIP-3 dexes are builder-deployed perp markets (e.g. `xyz` for stocks, `km` for indices/forex). Specify them with `--dex` or the `dex:coin` syntax:

```bash
# List available HIP-3 dexes
hl dexes

# Two ways to specify HIP-3 assets:
hl --dex xyz book TSLA            # --dex flag
hl book xyz:TSLA                  # dex:coin syntax

# HIP-3 market data
hl --dex xyz meta                 # Asset list for a dex
hl --dex xyz mids                 # Mid prices for a dex
hl --dex km mids                  # Kinetiq Markets mid prices
hl trades xyz:TSLA                # Recent trades
hl funding xyz:TSLA --start 1700000000000

# HIP-3 account state
hl --dex xyz state                # Margin/equity on xyz dex
hl --dex xyz positions            # Positions on xyz dex

# HIP-3 orders
hl --dex xyz order place TSLA buy --size 1 --price 400
hl --dex xyz order cancel TSLA --oid 123456
hl --dex xyz leverage set TSLA 5 --mode cross
```

## Output

Default output is human-readable tables. Use `--json` for machine-readable JSON.

```
$ hl book ETH

=== ETH Order Book ===

┌───────────┬──────────┐
│ Ask Price ┆ Ask Size │
╞═══════════╪══════════╡
│ 2135.5    ┆ 130.1075 │
│ 2135.4    ┆ 114.5266 │
│ 2135.3    ┆ 150.2073 │
└───────────┴──────────┘
  ---
┌───────────┬──────────┐
│ Bid Price ┆ Bid Size │
╞═══════════╪══════════╡
│ 2134.5    ┆ 82.3901  │
│ 2134.4    ┆ 0.0333   │
│ 2134.3    ┆ 8.5141   │
└───────────┴──────────┘
```

## AI Agent Integration

### Claude Code Skill

The skill is embedded in the `hl` binary and can be installed with:

```bash
hl install-skill              # installs to ~/.claude/commands/hl.md (all projects)
hl install-skill --project    # also installs to ./.claude/commands/hl.md (current project)
hl install-skill --force      # overwrite existing skill
```

Then invoke with `/hl` during a Claude Code session. The skill includes:
- Full command reference with examples
- Safety rules requiring confirmation before any write operations
- Workflow examples for common trading tasks
- `--json` flag usage for programmatic parsing

### Other AI Agents

Any AI agent that can execute shell commands can use `hl`. The `--json` flag provides machine-readable output for all commands:

```bash
# Read market data
hl mids --json          # All mid prices
hl book ETH --json      # Order book
hl positions --json     # Current positions

# Place orders (agent should confirm with user first)
hl order place ETH buy --size 0.1 --price 3000 --json
```

## Built With

- [hl-rs](https://github.com/kinetiq-research/hl-rs) — Hyperliquid Rust SDK
- [clap](https://crates.io/crates/clap) — CLI argument parsing
- [comfy-table](https://crates.io/crates/comfy-table) — Table formatting
- [alloy](https://crates.io/crates/alloy) — Ethereum signing
# hl-cli
