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

    /// Show L2 order book for a coin
    Book {
        /// Coin symbol (e.g. ETH, BTC)
        coin: String,
    },

    /// Show mid prices for all assets
    Mids,

    /// Show exchange metadata (asset list with indices)
    Meta,

    /// Show spot exchange metadata
    SpotMeta,

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
}

#[derive(Subcommand)]
pub enum OrderAction {
    /// Place a new order
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
