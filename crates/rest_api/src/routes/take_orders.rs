use crate::error::ApiError;
use rain_orderbook_common::raindex_client::take_orders::{
    TakeOrdersCalldataResult, TakeOrdersRequest,
};
use rain_orderbook_common::raindex_client::RaindexClient;
use rocket::serde::json::Json;
use rocket::{post, Route};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersApiRequest {
    pub yaml_content: String,
    #[serde(flatten)]
    pub request: TakeOrdersRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalApiResponse {
    pub token: String,
    pub spender: String,
    pub amount: String,
    pub formatted_amount: String,
    pub calldata: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TakeOrdersReadyResponse {
    pub orderbook: String,
    pub calldata: String,
    pub effective_price: String,
    pub prices: Vec<String>,
    pub expected_sell: String,
    pub max_sell_cap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status", content = "data")]
pub enum TakeOrdersApiResponse {
    NeedsApproval(ApprovalApiResponse),
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

#[post("/take-orders", data = "<request>")]
pub async fn take_orders(
    request: Json<TakeOrdersApiRequest>,
) -> Result<Json<TakeOrdersApiResponse>, ApiError> {
    let yaml_content = request.yaml_content.clone();
    let take_request = request.request.clone();

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
    use rain_orderbook_common::take_orders::TakeOrdersMode;

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
        assert_eq!(
            request.request.taker,
            "0x1111111111111111111111111111111111111111"
        );
        assert_eq!(request.request.chain_id, 1);
        assert_eq!(
            request.request.sell_token,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            request.request.buy_token,
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
        assert_eq!(request.request.amount, "100");
        assert_eq!(request.request.price_cap, "2.5");
        assert!(matches!(request.request.mode, TakeOrdersMode::BuyUpTo));
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

        assert!(matches!(request.request.mode, TakeOrdersMode::BuyExact));
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

        assert!(matches!(request.request.mode, TakeOrdersMode::SpendUpTo));
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

        assert!(matches!(request.request.mode, TakeOrdersMode::SpendExact));
    }

    #[test]
    fn test_ready_response_serialization() {
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
        assert!(json.contains("\"orderbook\":"));
        assert!(json.contains("\"calldata\":"));
        assert!(json.contains("\"effectivePrice\":"));
        assert!(json.contains("\"prices\":"));
        assert!(json.contains("\"expectedSell\":"));
        assert!(json.contains("\"maxSellCap\":"));
    }

    #[test]
    fn test_needs_approval_response_serialization() {
        let response = TakeOrdersApiResponse::NeedsApproval(ApprovalApiResponse {
            token: "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            spender: "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            amount: "1000".to_string(),
            formatted_amount: "1000".to_string(),
            calldata: "0xabcdef".to_string(),
        });

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"status\":\"needsApproval\""));
        assert!(json.contains("\"token\":"));
        assert!(json.contains("\"spender\":"));
        assert!(json.contains("\"amount\":"));
        assert!(json.contains("\"formattedAmount\":"));
        assert!(json.contains("\"calldata\":"));
    }
}
