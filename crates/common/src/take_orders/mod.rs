pub mod candidates;
pub mod config;
pub mod price;
pub mod simulation;

pub use candidates::{build_take_order_candidates_for_pair, TakeOrderCandidate};
pub use config::{
    build_take_orders_config_from_buy_simulation, BuiltTakeOrdersConfig, MinReceiveMode,
};
pub use price::cmp_float;
pub use simulation::{simulate_buy_over_candidates, SelectedTakeOrderLeg, SimulatedBuyResult};
