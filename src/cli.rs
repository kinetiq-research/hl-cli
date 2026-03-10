use clap::{Parser, Subcommand, ValueEnum};
use rust_decimal::Decimal;

#[derive(Parser)]
#[command(name = "hl", about = "Hyperliquid CLI", version)]
pub struct Cli {
    /// Network to use
    #[arg(long, value_enum, default_value = "mainnet", global = true, env = "HL_NETWORK")]
    pub network: Network,

    /// Output raw JSON instead of tables
    #[arg(long, global = true)]
    pub json: bool,

    /// HIP-3 builder-deployed perp dex (e.g. xyz, km, flx)
    #[arg(long, global = true, env = "HL_DEX")]
    pub dex: Option<String>,

    /// Skip confirmation prompts for write operations
    #[arg(long, short = 'y', global = true)]
    pub yes: bool,

    /// Watch mode: re-run every N seconds (for read commands)
    #[arg(long, short = 'w', global = true)]
    pub watch: Option<u64>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Network {
    Mainnet,
    Testnet,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show account state (margin, equity)
    State,

    /// Show open positions
    Positions,

    /// Show open orders
    Orders,

    /// Check status of a specific order
    OrderStatus {
        /// Order ID
        #[arg(long)]
        oid: u64,
    },

    /// Show recent fills
    Fills,

    /// Show historical orders
    HistoricalOrders,

    /// Show token balances
    Balance,

    /// Show L2 order book for a coin (with depth bars)
    Book {
        /// Coin symbol (e.g. ETH, BTC)
        coin: String,
        /// Number of levels to show (default 10)
        #[arg(long, default_value = "10")]
        levels: usize,
    },

    /// Show mid prices for all assets
    Mids,

    /// Show exchange metadata (asset list with indices)
    Meta,

    /// Show spot exchange metadata
    SpotMeta,

    /// List HIP-3 builder-deployed perp dexes
    Dexes,

    /// Install the Claude Code skill (~/.claude/commands/hl.md)
    InstallSkill {
        /// Also install into the current project (.claude/commands/hl.md)
        #[arg(long)]
        project: bool,
        /// Overwrite existing skill file
        #[arg(long)]
        force: bool,
    },

    /// Initialize ~/.hl.env and install the Claude Code skill
    Init {
        /// Private key (API Wallet key from Hyperliquid UI)
        #[arg(long)]
        private_key: Option<String>,
        /// Wallet address for read-only operations
        #[arg(long)]
        address: Option<String>,
        /// Default network
        #[arg(long, value_enum)]
        network: Option<Network>,
        /// Overwrite existing files
        #[arg(long)]
        force: bool,
        /// Skip installing the Claude Code skill
        #[arg(long)]
        no_skill: bool,
    },

    /// Show funding rate history for a coin
    Funding {
        /// Coin symbol
        coin: String,
        /// Start time (unix ms)
        #[arg(long)]
        start: u64,
        /// End time (unix ms)
        #[arg(long)]
        end: Option<u64>,
    },

    /// Show user funding payments
    UserFunding {
        /// Start time (unix ms)
        #[arg(long)]
        start: u64,
        /// End time (unix ms)
        #[arg(long)]
        end: Option<u64>,
    },

    /// Show candle data
    Candles {
        /// Coin symbol
        coin: String,
        /// Interval (e.g. 1m, 5m, 15m, 1h, 4h, 1d)
        #[arg(long, default_value = "1h")]
        interval: String,
        /// Start time (unix ms)
        #[arg(long)]
        start: u64,
        /// End time (unix ms)
        #[arg(long)]
        end: u64,
    },

    /// Show recent trades for a coin
    Trades {
        /// Coin symbol
        coin: String,
    },

    /// Order management
    Order {
        #[command(subcommand)]
        action: OrderAction,
    },

    /// Leverage management
    Leverage {
        #[command(subcommand)]
        action: LeverageAction,
    },

    /// Transfer USDC to another Hyperliquid address
    Transfer {
        /// Destination address (0x...)
        #[arg(long)]
        to: String,
        /// Amount in USDC
        #[arg(long)]
        amount: Decimal,
    },

    /// Withdraw USDC to L1
    Withdraw {
        /// Destination address (0x...)
        #[arg(long)]
        to: String,
        /// Amount in USDC
        #[arg(long)]
        amount: Decimal,
    },

    // === NEW COMMANDS ===

    /// Check API health / connectivity status
    Status,

    /// Show bid-ask spread for a coin
    Spread {
        /// Coin symbol (e.g. ETH, BTC)
        coin: String,
    },

    /// Show PnL summary across all positions
    Pnl,

    /// Show open interest for a coin or all coins
    Oi {
        /// Coin symbol (optional, shows all if omitted)
        coin: Option<String>,
    },

    /// Search/filter assets by name
    Search {
        /// Search query (case-insensitive substring match)
        query: String,
    },

    /// Interactive REPL shell
    Shell,

    /// Self-upgrade hl binary from source
    Upgrade,
}

#[derive(Subcommand)]
pub enum OrderAction {
    /// Place a new limit order
    Place {
        /// Coin symbol (e.g. ETH, BTC)
        coin: String,
        /// Side: buy or sell
        #[arg(value_enum)]
        side: Side,
        /// Order size
        #[arg(long)]
        size: Decimal,
        /// Limit price
        #[arg(long)]
        price: Decimal,
        /// Time-in-force
        #[arg(long, value_enum, default_value = "gtc")]
        tif: TifArg,
        /// Reduce-only
        #[arg(long)]
        reduce_only: bool,
        /// Client order ID
        #[arg(long)]
        cloid: Option<String>,
        /// Trigger price (makes this a stop/tp order)
        #[arg(long)]
        trigger_price: Option<Decimal>,
        /// Trigger type (required if trigger-price set)
        #[arg(long, value_enum)]
        trigger_type: Option<TriggerType>,
        /// Whether trigger executes as market order
        #[arg(long, default_value = "true")]
        trigger_is_market: bool,
    },

    /// Place a market order (IOC at slippage-adjusted price)
    Market {
        /// Coin symbol (e.g. ETH, BTC)
        coin: String,
        /// Side: buy or sell
        #[arg(value_enum)]
        side: Side,
        /// Order size in coin units
        #[arg(long, group = "sizing")]
        size: Option<Decimal>,
        /// Order size in USD notional
        #[arg(long, group = "sizing")]
        amount: Option<Decimal>,
        /// Slippage tolerance as percentage (default 1.0 = 1%)
        #[arg(long, default_value = "1.0")]
        slippage: Decimal,
        /// Reduce-only
        #[arg(long)]
        reduce_only: bool,
    },

    /// Place batch orders from JSON
    Batch {
        /// JSON array of orders: [{"coin":"ETH","side":"buy","size":"0.1","price":"3000"}, ...]
        /// Or path to a JSON file (prefix with @)
        orders_json: String,
    },

    /// Cancel an order by order ID
    Cancel {
        /// Coin symbol
        coin: String,
        /// Order ID
        #[arg(long)]
        oid: u64,
    },

    /// Cancel an order by client order ID
    CancelByCloid {
        /// Coin symbol
        coin: String,
        /// Client order ID
        #[arg(long)]
        cloid: String,
    },

    /// Cancel all open orders (optionally for a specific coin)
    CancelAll {
        /// Coin symbol (optional - cancels all if omitted)
        coin: Option<String>,
    },

    /// Modify an existing order
    Modify {
        /// Order ID to modify
        #[arg(long)]
        oid: u64,
        /// Coin symbol
        coin: String,
        /// New side
        #[arg(value_enum)]
        side: Side,
        /// New size
        #[arg(long)]
        size: Decimal,
        /// New price
        #[arg(long)]
        price: Decimal,
        /// Time-in-force
        #[arg(long, value_enum, default_value = "gtc")]
        tif: TifArg,
        /// Reduce-only
        #[arg(long)]
        reduce_only: bool,
        /// Client order ID
        #[arg(long)]
        cloid: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum LeverageAction {
    /// Set leverage for a coin
    Set {
        /// Coin symbol
        coin: String,
        /// Leverage value
        leverage: u32,
        /// Margin mode
        #[arg(long, value_enum, default_value = "cross")]
        mode: MarginMode,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum TifArg {
    Gtc,
    Ioc,
    Alo,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum TriggerType {
    Tp,
    Sl,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum MarginMode {
    Cross,
    Isolated,
}
