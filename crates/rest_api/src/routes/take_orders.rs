use crate::error::ApiError;
use rain_orderbook_common::raindex_client::take_orders::TakeOrdersRequest;
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
pub struct TakeOrdersApiResponse {
    pub orderbook: String,
    pub calldata: String,
    pub effective_price: String,
    pub prices: Vec<String>,
}

async fn execute_take_orders(
    yaml_content: String,
    request: TakeOrdersRequest,
) -> Result<TakeOrdersApiResponse, ApiError> {
    let client = RaindexClient::new(vec![yaml_content], None)?;

    let result = client.get_take_orders_calldata(request).await?;

    let effective_price = result.effective_price().format().map_err(|e| {
        ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
            e,
        ))
    })?;

    let prices: Result<Vec<String>, _> = result
        .prices()
        .iter()
        .map(|p| {
            p.format().map_err(|e| {
                ApiError::Raindex(rain_orderbook_common::raindex_client::RaindexError::Float(
                    e,
                ))
            })
        })
        .collect();

    Ok(TakeOrdersApiResponse {
        orderbook: result.orderbook().to_string(),
        calldata: result.calldata().to_string(),
        effective_price,
        prices: prices?,
    })
}

#[post("/take-orders", format = "json", data = "<request>")]
pub async fn take_orders(
    request: Json<TakeOrdersApiRequest>,
) -> Result<Json<TakeOrdersApiResponse>, ApiError> {
    let yaml_content = request.yaml_content.clone();
    let take_request = request.request.clone();

    let response = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                ApiError::Raindex(
                    rain_orderbook_common::raindex_client::RaindexError::JsError(format!(
                        "Failed to create runtime: {}",
                        e
                    )),
                )
            })?;

        rt.block_on(execute_take_orders(yaml_content, take_request))
    })
    .await
    .map_err(|e| {
        ApiError::Raindex(
            rain_orderbook_common::raindex_client::RaindexError::JsError(e.to_string()),
        )
    })??;

    Ok(Json(response))
}

pub fn routes() -> Vec<Route> {
    rocket::routes![take_orders]
}

#[cfg(test)]
mod tests {
    use super::*;
    use rain_orderbook_common::take_orders::MinReceiveMode;

    #[test]
    fn test_request_deserialization() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 1,
            "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "buyAmount": "100",
            "priceCap": "2.5",
            "minReceiveMode": "partial"
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
        assert_eq!(request.request.buy_amount, "100");
        assert_eq!(request.request.price_cap, "2.5");
        assert!(matches!(
            request.request.min_receive_mode,
            MinReceiveMode::Partial
        ));
    }

    #[test]
    fn test_request_deserialization_exact_mode() {
        let json = r#"{
            "yamlContent": "version: 1",
            "taker": "0x1111111111111111111111111111111111111111",
            "chainId": 137,
            "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "buyAmount": "50.5",
            "priceCap": "1.0",
            "minReceiveMode": "exact"
        }"#;

        let request: TakeOrdersApiRequest = serde_json::from_str(json).unwrap();

        assert!(matches!(
            request.request.min_receive_mode,
            MinReceiveMode::Exact
        ));
    }

    #[test]
    fn test_response_serialization() {
        let response = TakeOrdersApiResponse {
            orderbook: "0x1234567890123456789012345678901234567890".to_string(),
            calldata: "0xabcdef".to_string(),
            effective_price: "1.5".to_string(),
            prices: vec!["1.4".to_string(), "1.6".to_string()],
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"orderbook\":"));
        assert!(json.contains("\"calldata\":"));
        assert!(json.contains("\"effectivePrice\":"));
        assert!(json.contains("\"prices\":"));
    }
}
