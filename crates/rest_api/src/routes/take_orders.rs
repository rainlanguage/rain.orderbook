use crate::error::{ApiError, ApiErrorResponse};
use rain_orderbook_common::raindex_client::take_orders::{
    TakeOrdersCalldataResult, TakeOrdersRequest,
};
use rain_orderbook_common::raindex_client::RaindexClient;
use rocket::serde::json::Json;
use rocket::{post, Route};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum TakeOrdersMode {
    BuyExact,
    BuyUpTo,
    SpendExact,
    SpendUpTo,
}

impl From<TakeOrdersMode> for rain_orderbook_common::take_orders::TakeOrdersMode {
    fn from(mode: TakeOrdersMode) -> Self {
        match mode {
            TakeOrdersMode::BuyExact => Self::BuyExact,
            TakeOrdersMode::BuyUpTo => Self::BuyUpTo,
            TakeOrdersMode::SpendExact => Self::SpendExact,
            TakeOrdersMode::SpendUpTo => Self::SpendUpTo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersApiRequest {
    #[schema(
        example = "networks:\n  base:\n    rpc: https://mainnet.base.org\n    chain-id: 8453\nsubgraphs:\n  base: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn\norderbooks:\n  base:\n    address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7\n    network: base\n    subgraph: base"
    )]
    pub yaml_content: String,
    #[schema(example = "0x1111111111111111111111111111111111111111")]
    pub taker: String,
    #[schema(example = 8453)]
    pub chain_id: u32,
    #[schema(example = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")]
    pub sell_token: String,
    #[schema(example = "0x4200000000000000000000000000000000000006")]
    pub buy_token: String,
    pub mode: TakeOrdersMode,
    #[schema(example = "1000")]
    pub amount: String,
    #[schema(example = "0.0005")]
    pub price_cap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "token": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
    "spender": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
    "amount": "1000",
    "formattedAmount": "1000",
    "calldata": "0x095ea7b3..."
}))]
pub struct ApprovalApiResponse {
    #[schema(example = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913")]
    pub token: String,
    #[schema(example = "0xd2938e7c9fe3597f78832ce780feb61945c377d7")]
    pub spender: String,
    #[schema(example = "1000")]
    pub amount: String,
    #[schema(example = "1000")]
    pub formatted_amount: String,
    #[schema(example = "0x095ea7b3...")]
    pub calldata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "orderbook": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
    "calldata": "0x...",
    "effectivePrice": "0.00045",
    "prices": ["0.00044", "0.00046"],
    "expectedSell": "450",
    "maxSellCap": "500"
}))]
pub struct TakeOrdersReadyResponse {
    #[schema(example = "0xd2938e7c9fe3597f78832ce780feb61945c377d7")]
    pub orderbook: String,
    #[schema(example = "0x...")]
    pub calldata: String,
    #[schema(example = "0.00045")]
    pub effective_price: String,
    #[schema(example = json!(["0.00044", "0.00046"]))]
    pub prices: Vec<String>,
    #[schema(example = "450")]
    pub expected_sell: String,
    #[schema(example = "500")]
    pub max_sell_cap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", tag = "status", content = "data")]
pub enum TakeOrdersApiResponse {
    #[schema(title = "NeedsApproval")]
    NeedsApproval(ApprovalApiResponse),
    #[schema(title = "Ready")]
    Ready(TakeOrdersReadyResponse),
}

async fn execute_take_orders(
    yaml_content: String,
    request: TakeOrdersRequest,
) -> Result<TakeOrdersApiResponse, ApiError> {
    let client = RaindexClient::new(vec![yaml_content], None)?;

    let result = client.get_take_orders_calldata(request).await?;

    match result {
        TakeOrdersCalldataResult::NeedsApproval(approval_info) => {
            let amount = approval_info.amount.format().map_err(|e| {
                ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                    e,
                ))
            })?;

            Ok(TakeOrdersApiResponse::NeedsApproval(ApprovalApiResponse {
                token: approval_info.token.to_string(),
                spender: approval_info.spender.to_string(),
                amount,
                formatted_amount: approval_info.formatted_amount,
                calldata: approval_info.calldata.to_string(),
            }))
        }
        TakeOrdersCalldataResult::Ready(take_orders_info) => {
            let effective_price = take_orders_info.effective_price.format().map_err(|e| {
                ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                    e,
                ))
            })?;

            let prices: Result<Vec<String>, _> = take_orders_info
                .prices
                .iter()
                .map(|p| {
                    p.format().map_err(|e| {
                        ApiError::Raindex(
                            rain_orderbook_common::raindex_client::RaindexError::Float(e),
                        )
                    })
                })
                .collect();

            let expected_sell = take_orders_info.expected_sell.format().map_err(|e| {
                ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                    e,
                ))
            })?;

            let max_sell_cap = take_orders_info.max_sell_cap.format().map_err(|e| {
                ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                    e,
                ))
            })?;

            Ok(TakeOrdersApiResponse::Ready(TakeOrdersReadyResponse {
                orderbook: take_orders_info.orderbook.to_string(),
                calldata: take_orders_info.calldata.to_string(),
                effective_price,
                prices: prices?,
                expected_sell,
                max_sell_cap,
            }))
        }
    }
}

#[utoipa::path(
    post,
    path = "/take-orders",
    tag = "Take Orders",
    request_body = TakeOrdersApiRequest,
    responses(
        (status = 200, description = "Successfully processed take orders request. Returns either approval info if token approval is needed, or ready calldata if approval is sufficient.", body = TakeOrdersApiResponse,
            examples(
                ("Ready" = (
                    summary = "Calldata ready to execute",
                    description = "Returned when the taker has sufficient token approval. The calldata can be submitted directly to the orderbook.",
                    value = json!({
                        "status": "ready",
                        "data": {
                            "orderbook": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
                            "calldata": "0x...",
                            "effectivePrice": "0.00045",
                            "prices": ["0.00044", "0.00046"],
                            "expectedSell": "450",
                            "maxSellCap": "500"
                        }
                    })
                )),
                ("NeedsApproval" = (
                    summary = "Token approval required",
                    description = "Returned when the taker needs to approve token spending before executing. Submit the approval calldata first, then retry the request.",
                    value = json!({
                        "status": "needsApproval",
                        "data": {
                            "token": "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
                            "spender": "0xd2938e7c9fe3597f78832ce780feb61945c377d7",
                            "amount": "1000",
                            "formattedAmount": "1000",
                            "calldata": "0x095ea7b3..."
                        }
                    })
                ))
            )
        ),
        (status = 400, description = "Invalid request parameters", body = ApiErrorResponse),
        (status = 404, description = "No liquidity found or configuration not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    )
)]
#[post("/take-orders", data = "<request>")]
pub async fn take_orders(
    request: Json<TakeOrdersApiRequest>,
) -> Result<Json<TakeOrdersApiResponse>, ApiError> {
    let yaml_content = request.yaml_content.clone();
    let take_request = TakeOrdersRequest {
        taker: request.taker.clone(),
        chain_id: request.chain_id,
        sell_token: request.sell_token.clone(),
        buy_token: request.buy_token.clone(),
        mode: request.mode.into(),
        amount: request.amount.clone(),
        price_cap: request.price_cap.clone(),
    };

    // RaindexClient contains Rc<RefCell<...>> which is not Send, but Rocket requires
    // Send futures. We use spawn_blocking with a dedicated runtime to run everything
    // on a single thread where Rc<RefCell> is safe.
    let response = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| ApiError::Internal(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(execute_take_orders(yaml_content, take_request))
    })
    .await
    .map_err(|e| ApiError::Internal(format!("Task execution failed: {}", e)))??;

    Ok(Json(response))
}

pub fn routes() -> Vec<Route> {
    rocket::routes![take_orders]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_deserialization_buy_up_to() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 1,
            "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "mode": "buyUpTo",
            "amount": "100",
            "priceCap": "2.5"
        }"#;

        let request: TakeOrdersApiRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.yaml_content, "version: 1");
        assert_eq!(request.taker, "0x1111111111111111111111111111111111111111");
        assert_eq!(request.chain_id, 1);
        assert_eq!(
            request.sell_token,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            request.buy_token,
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        assert_eq!(request.amount, "100");
        assert_eq!(request.price_cap, "2.5");
        assert!(matches!(request.mode, TakeOrdersMode::BuyUpTo));
    }

    #[test]
    fn test_request_deserialization_buy_exact() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 137,
            "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "mode": "buyExact",
            "amount": "50.5",
            "priceCap": "1.0"
        }"#;

        let request: TakeOrdersApiRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(request.mode, TakeOrdersMode::BuyExact));
    }

    #[test]
    fn test_request_deserialization_spend_up_to() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 1,
            "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "mode": "spendUpTo",
            "amount": "100",
            "priceCap": "2.5"
        }"#;

        let request: TakeOrdersApiRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(request.mode, TakeOrdersMode::SpendUpTo));
    }

    #[test]
    fn test_request_deserialization_spend_exact() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 1,
            "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "mode": "spendExact",
            "amount": "75",
            "priceCap": "3.0"
        }"#;

        let request: TakeOrdersApiRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(request.mode, TakeOrdersMode::SpendExact));
    }

    #[test]
    fn test_response_serialization_ready() {
        let response = TakeOrdersApiResponse::Ready(TakeOrdersReadyResponse {
            orderbook: "0x1234567890123456789012345678901234567890".to_string(),
            calldata: "0xabcdef".to_string(),
            effective_price: "1.5".to_string(),
            prices: vec!["1.4".to_string(), "1.6".to_string()],
            expected_sell: "150".to_string(),
            max_sell_cap: "200".to_string(),
        });

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"status\":\"ready\""));
        assert!(json.contains("\"data\":"));
        assert!(json.contains("\"orderbook\":"));
        assert!(json.contains("\"calldata\":"));
        assert!(json.contains("\"effectivePrice\":"));
        assert!(json.contains("\"prices\":"));
        assert!(json.contains("\"expectedSell\":"));
        assert!(json.contains("\"maxSellCap\":"));
    }

    #[test]
    fn test_response_serialization_needs_approval() {
        let response = TakeOrdersApiResponse::NeedsApproval(ApprovalApiResponse {
            token: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
            spender: "0xd2938e7c9fe3597f78832ce780feb61945c377d7".to_string(),
            amount: "1000".to_string(),
            formatted_amount: "1000".to_string(),
            calldata: "0x095ea7b3...".to_string(),
        });

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"status\":\"needsApproval\""));
        assert!(json.contains("\"data\":"));
        assert!(json.contains("\"token\":"));
        assert!(json.contains("\"spender\":"));
        assert!(json.contains("\"amount\":"));
        assert!(json.contains("\"formattedAmount\":"));
        assert!(json.contains("\"calldata\":"));
    }
}
