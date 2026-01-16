mod error;
mod routes;

use error::ApiErrorResponse;
use rocket::http::Method;
use rocket::{launch, Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};
use routes::take_orders::{TakeOrdersApiRequest, TakeOrdersApiResponse, TakeOrdersMode};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rain Orderbook API",
        description = "REST API for interacting with Rain Orderbook."
    ),
    paths(routes::take_orders::take_orders),
    components(schemas(
        TakeOrdersApiRequest,
        TakeOrdersApiResponse,
        TakeOrdersMode,
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
    fn test_cors_preflight() {
        let client = client();
        let response = client
            .options("/take-orders")
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
    fn test_take_orders_missing_yaml() {
        let client = client();
        let response = client
            .post("/take-orders")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "mode": "buyUpTo",
                "amount": "100",
                "priceCap": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_take_orders_invalid_address() {
        let client = client();
        let response = client
            .post("/take-orders")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "invalid-address",
                "chainId": 1,
                "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "mode": "buyUpTo",
                "amount": "100",
                "priceCap": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_take_orders_same_token_pair() {
        let client = client();
        let response = client
            .post("/take-orders")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "buyToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "mode": "buyUpTo",
                "amount": "100",
                "priceCap": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_take_orders_zero_amount() {
        let client = client();
        let response = client
            .post("/take-orders")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "mode": "buyUpTo",
                "amount": "0",
                "priceCap": "2.5"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_take_orders_negative_price_cap() {
        let client = client();
        let response = client
            .post("/take-orders")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "buyToken": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "mode": "buyUpTo",
                "amount": "100",
                "priceCap": "-1"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_take_orders_missing_field() {
        let client = client();
        let response = client
            .post("/take-orders")
            .header(ContentType::JSON)
            .body(
                r#"{
                "yamlContent": "version: 1",
                "taker": "0x1111111111111111111111111111111111111111",
                "chainId": 1,
                "sellToken": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "mode": "buyUpTo",
                "amount": "100",
                "priceCap": "2.5"
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
    fn test_openapi_json_contains_take_orders_path() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert!(spec["paths"]["/take-orders"]["post"].is_object());
        assert_eq!(
            spec["paths"]["/take-orders"]["post"]["tags"][0],
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
        assert!(schemas["TakeOrdersApiRequest"].is_object());
        assert!(schemas["TakeOrdersApiResponse"].is_object());
        assert!(schemas["TakeOrdersMode"].is_object());
        assert!(schemas["ApiErrorResponse"].is_object());
    }

    #[test]
    fn test_openapi_json_contains_response_codes() {
        let client = client();
        let response = client.get("/swagger/openapi.json").dispatch();
        let body = response.into_string().unwrap();
        let spec: serde_json::Value = serde_json::from_str(&body).unwrap();

        let responses = &spec["paths"]["/take-orders"]["post"]["responses"];
        assert!(responses["200"].is_object());
        assert!(responses["400"].is_object());
        assert!(responses["404"].is_object());
        assert!(responses["500"].is_object());
    }
}
