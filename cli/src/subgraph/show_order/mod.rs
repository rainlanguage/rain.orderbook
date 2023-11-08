use bigdecimal::BigDecimal;
use ethers::types::Bytes;
use ethers::types::H160;
use ethers::types::U256;
use ethers::utils::format_units;
use graphql_client::GraphQLQuery;
use graphql_client::Response;
use rust_bigint::BigInt;

use std::str::FromStr;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/subgraph/schema/orders.schema.json",
    query_path = "src/subgraph/queries/orders.graphql",
    response_derives = "Debug, Serialize, Deserialize"
)]
pub struct OrdersQuery;

/// Struct representing the OrderVaults to be displayed via the [view_orders](crate::cli::listorders::listorders::view_orders) function.
/// This is intended to be used by the [view_orders](crate::cli::listorders::listorders::view_orders) function within the crate itself.
/// Use of struct is outside of the crate is unlikely and is not recommended.
#[derive(Clone, Debug)]
pub struct OrderVaults {
    pub token: String,
    pub balance: String,
}

/// Struct representing the TakeOrders to be displayed via the [view_orders](crate::cli::listorders::listorders::view_orders) function.
/// This is intended to be used by the [view_orders](crate::cli::listorders::listorders::view_orders) function within the crate itself.
/// Use of struct is outside of the crate is unlikely and is not recommended.
#[derive(Clone, Debug)]
pub struct TakeOrderDetails {
    pub input_token: String,
    pub input_amount: String,
    pub output_token: String,
    pub output_amount: String,
    pub transaction_id: String,
}

/// Struct representing the Order to be displayed via the [view_orders](crate::cli::listorders::listorders::view_orders) function.
/// This is intended to be used by the [view_orders](crate::cli::listorders::listorders::view_orders) function within the crate itself.
/// Use of struct is outside of the crate is unlikely and is not recommended.
#[derive(Clone, Debug)]
pub struct OrdersDetails {
    pub id: String,
    pub owner: String,
    pub input_vaults: Vec<OrderVaults>,
    pub output_vaults: Vec<OrderVaults>,
    pub take_orders: Vec<TakeOrderDetails>,
}

/// Fetches the order from the subgraph and build the a vector of [OrdersDetails] struct for the orders to be displayed
/// via the [view_orders](crate::cli::listorders::listorders::view_orders) function.
/// This is intended to be used by the [view_orders](crate::cli::listorders::listorders::view_orders) function within the crate itself.
///
/// # Arguments
/// * - `sg_uri`: Subgraph api endpoint.
pub async fn get_order_details_display(sg_uri: String) -> anyhow::Result<Vec<OrdersDetails>> {
    let variables = orders_query::Variables {};
    let request_body = OrdersQuery::build_query(variables);
    let client = reqwest::Client::new();
    let res = client
        .post(sg_uri.clone())
        .json(&request_body)
        .send()
        .await?;
    let response_body: Response<orders_query::ResponseData> = res.json().await?;

    let orders: Vec<OrdersDetails> = response_body
        .data
        .unwrap()
        .orders
        .iter()
        .map(|order| {
            let order_owner = &*order.owner.id;
            let order_owner = hex::encode(order_owner.to_vec());
            let order_owner = H160::from_str(order_owner.as_str()).unwrap();
            let order_owner = format!("{order_owner:#020x}");

            let ip_io: Vec<OrderVaults> = order
                .valid_inputs
                .as_ref()
                .unwrap()
                .iter()
                .map(|x| {
                    let token_symbol = &x.token.symbol;
                    let vault_balance =
                        U256::from_dec_str(&x.token_vault.balance.to_str_radix(16)).unwrap();
                    let token_decimals = u32::from_str(&x.token.decimals.to_string()).unwrap();
                    let token_amount = format_units(vault_balance, token_decimals)
                        .unwrap()
                        .to_string();

                    OrderVaults {
                        token: token_symbol.to_string(),
                        balance: token_amount,
                    }
                })
                .collect();

            let op_io: Vec<OrderVaults> = order
                .valid_outputs
                .as_ref()
                .unwrap()
                .iter()
                .map(|x| {
                    let token_symbol = &x.token.symbol;
                    let vault_balance =
                        U256::from_dec_str(&x.token_vault.balance.to_str_radix(16)).unwrap();
                    let token_decimals = u32::from_str(&x.token.decimals.to_string()).unwrap();
                    let token_amount = format_units(vault_balance, token_decimals)
                        .unwrap()
                        .to_string();

                    OrderVaults {
                        token: token_symbol.to_string(),
                        balance: token_amount,
                    }
                })
                .collect();

            let take_orders: Vec<TakeOrderDetails> = order
                .take_orders
                .as_ref()
                .unwrap()
                .iter()
                .map(|t| TakeOrderDetails {
                    input_token: t.input_token.symbol.clone(),
                    input_amount: t.input_display.to_string(),
                    output_token: t.output_token.symbol.clone(),
                    output_amount: t.output_display.to_string(),
                    transaction_id: t.transaction.id.clone(),
                })
                .collect();

            OrdersDetails {
                id: order.id.clone(),
                owner: order_owner,
                input_vaults: ip_io,
                output_vaults: op_io,
                take_orders: take_orders,
            }
        })
        .collect();

    Ok(orders)
}
