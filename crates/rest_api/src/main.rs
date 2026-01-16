mod error;
mod routes;

use error::ApiErrorResponse;
use rocket::http::Method;
use rocket::{launch, Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use routes::take_orders::{BuyRequest, SellRequest, TakeOrdersApiResponse};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rain Orderbook API",
        description = "REST API for interacting with Rain Orderbook."
    ),
    paths(routes::take_orders::buy, routes::take_orders::sell),
    components(schemas(
        BuyRequest,
        SellRequest,
        TakeOrdersApiResponse,
        ApiErrorResponse
    )),
    tags(
        (name = "Take Orders", description = "Endpoints for generating take orders calldata")
    )
)]
struct ApiDoc;

fn configure_cors() -> CorsOptions {
    CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![Method::Get, Method::Post, Method::Options]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::all(),
        ..Default::default()
    }
}

fn rocket() -> Rocket<Build> {
    let cors = configure_cors()
        .to_cors()
        .expect("CORS configuration failed");

    rocket::build()
        .attach(cors.clone())
        .mount("/", routes::take_orders::routes())
        .mount("/", rocket_cors::catch_all_options_routes())
        .mount(
            "/",
            SwaggerUi::new("/swagger/<tail..>").url("/swagger/openapi.json", ApiDoc::openapi()),
        )
        .manage(cors)
}

#[launch]
fn launch() -> Rocket<Build> {
    rocket()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;

    fn client() -> Client {
        Client::tracked(rocket()).expect("valid rocket instance")
    }

    #[test]
    fn test_cors_preflight_buy() {
        let client = client();
        let response = client
            .options("/take-orders/buy")
            .header(rocket::http::Header::new(
                "Access-Control-Request-Method",
                "POST",
            ))
            .header(rocket::http::Header::new(
                "Access-Control-Request-Headers",
                "content-type",
            ))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_cors_preflight_sell() {
        let client = client();
        let response = client
            .options("/take-orders/sell")
            .header(rocket::http::Header::new(
                "Access-Control-Request-Method",
                "POST",
            ))
            .header(rocket::http::Header::new(
                "Access-Control-Request-Headers",
                "content-type",
            ))
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_buy_missing_yaml() {
        let client = client();
        let response = client
            .post("/take-orders/buy")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_buy_invalid_address() {
        let client = client();
        let response = client
            .post("/take-orders/buy")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "invalid-address",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_buy_same_token_pair() {
        let client = client();
        let response = client
            .post("/take-orders/buy")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_buy_zero_amount() {
        let client = client();
        let response = client
            .post("/take-orders/buy")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "0",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_buy_negative_max_ratio() {
        let client = client();
        let response = client
            .post("/take-orders/buy")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "100",
                "maxRatio": "-1"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_buy_missing_field() {
        let client = client();
        let response = client
            .post("/take-orders/buy")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[test]
    fn test_sell_missing_yaml() {
        let client = client();
        let response = client
            .post("/take-orders/sell")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_sell_same_token_pair() {
        let client = client();
        let response = client
            .post("/take-orders/sell")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_sell_invalid_address() {
        let client = client();
        let response = client
            .post("/take-orders/sell")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "invalid-address",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_sell_zero_amount() {
        let client = client();
        let response = client
            .post("/take-orders/sell")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "0",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_sell_negative_max_ratio() {
        let client = client();
        let response = client
            .post("/take-orders/sell")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "tokenOut": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "amount": "100",
                "maxRatio": "-1"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_sell_missing_field() {
        let client = client();
        let response = client
            .post("/take-orders/sell")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "tokenIn": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "amount": "100",
                "maxRatio": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[test]
    fn test_swagger_ui_returns_html() {
        let client = client();
        let response = client.get("/swagger/").dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        assert!(body.contains("<!DOCTYPE html>"));
        assert!(body.contains("swagger-ui"));
    }

    #[test]
    fn test_openapi_json_returns_valid_spec() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();

        assert_eq!(response.status(), Status::Ok);
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert_eq!(spec["openapi"], "3.1.0");
        assert_eq!(spec["info"]["title"], "Rain Orderbook API");
    }

    #[test]
    fn test_openapi_json_contains_buy_and_sell_paths() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert!(spec["paths"]["/take-orders/buy"]["post"].is_object());
        assert_eq!(
            spec["paths"]["/take-orders/buy"]["post"]["tags"][0],
            "Take Orders"
        );

        assert!(spec["paths"]["/take-orders/sell"]["post"].is_object());
        assert_eq!(
            spec["paths"]["/take-orders/sell"]["post"]["tags"][0],
            "Take Orders"
        );
    }

    #[test]
    fn test_openapi_json_contains_schemas() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        let schemas = &spec["components"]["schemas"];
        assert!(schemas["BuyRequest"].is_object());
        assert!(schemas["SellRequest"].is_object());
        assert!(schemas["TakeOrdersApiResponse"].is_object());
        assert!(schemas["ApiErrorResponse"].is_object());
    }

    #[test]
    fn test_openapi_json_contains_response_codes() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        let buy_responses = &spec["paths"]["/take-orders/buy"]["post"]["responses"];
        assert!(buy_responses["200"].is_object());
        assert!(buy_responses["400"].is_object());
        assert!(buy_responses["404"].is_object());
        assert!(buy_responses["500"].is_object());

        let sell_responses = &spec["paths"]["/take-orders/sell"]["post"]["responses"];
        assert!(sell_responses["200"].is_object());
        assert!(sell_responses["400"].is_object());
        assert!(sell_responses["404"].is_object());
        assert!(sell_responses["500"].is_object());
    }

    #[test]
    fn test_openapi_buy_request_field_descriptions() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        let buy_schema = &spec["components"]["schemas"]["BuyRequest"]["properties"];

        assert_eq!(
            buy_schema["yamlContent"]["description"],
            "YAML configuration containing network RPC endpoints, subgraph URLs, and orderbook addresses"
        );
        assert_eq!(
            buy_schema["taker"]["description"],
            "Address that will execute the transaction"
        );
        assert_eq!(
            buy_schema["chainId"]["description"],
            "Chain ID where the trade will be executed"
        );
        assert_eq!(
            buy_schema["tokenIn"]["description"],
            "Token address you are giving (spending)"
        );
        assert_eq!(
            buy_schema["tokenOut"]["description"],
            "Token address you are receiving (buying)"
        );
        assert_eq!(
            buy_schema["amount"]["description"],
            "Amount of tokenOut to receive (human-readable decimal string)"
        );
        assert_eq!(
            buy_schema["maxRatio"]["description"],
            "Maximum price ratio (tokenIn per 1 tokenOut). Trade fails if actual ratio exceeds this."
        );
        assert_eq!(
            buy_schema["exact"]["description"],
            "If true, transaction reverts unless exactly the specified amount is received. If false (default), receives up to the specified amount."
        );
    }

    #[test]
    fn test_openapi_sell_request_field_descriptions() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        let sell_schema = &spec["components"]["schemas"]["SellRequest"]["properties"];

        assert_eq!(
            sell_schema["yamlContent"]["description"],
            "YAML configuration containing network RPC endpoints, subgraph URLs, and orderbook addresses"
        );
        assert_eq!(
            sell_schema["taker"]["description"],
            "Address that will execute the transaction"
        );
        assert_eq!(
            sell_schema["chainId"]["description"],
            "Chain ID where the trade will be executed"
        );
        assert_eq!(
            sell_schema["tokenIn"]["description"],
            "Token address you are giving (selling)"
        );
        assert_eq!(
            sell_schema["tokenOut"]["description"],
            "Token address you are receiving"
        );
        assert_eq!(
            sell_schema["amount"]["description"],
            "Amount of tokenIn to spend (human-readable decimal string)"
        );
        assert_eq!(
            sell_schema["maxRatio"]["description"],
            "Maximum price ratio (tokenIn per 1 tokenOut). Trade fails if actual ratio exceeds this."
        );
        assert_eq!(
            sell_schema["exact"]["description"],
            "If true, transaction reverts unless exactly the specified amount is spent. If false (default), spends up to the specified amount."
        );
    }

    #[test]
    fn test_openapi_response_field_descriptions() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        let response_schema = &spec["components"]["schemas"]["TakeOrdersApiResponse"]["properties"];

        assert_eq!(
            response_schema["orderbook"]["description"],
            "Address of the orderbook contract to call"
        );
        assert_eq!(
            response_schema["calldata"]["description"],
            "ABI-encoded calldata for the takeOrders4 function"
        );
        assert_eq!(
            response_schema["effectivePrice"]["description"],
            "Blended effective price across all selected orders (tokenIn per 1 tokenOut)"
        );
        assert_eq!(
            response_schema["prices"]["description"],
            "Individual prices for each order leg, sorted from best to worst"
        );
        assert_eq!(
            response_schema["expectedSell"]["description"],
            "Expected amount of tokenIn to spend based on current quotes"
        );
        assert_eq!(
            response_schema["maxSellCap"]["description"],
            "Maximum tokenIn that could be spent (worst-case based on maxRatio)"
        );
    }
}
