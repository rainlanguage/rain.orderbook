mod error;
mod routes;

use rocket::{Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

fn create_app() -> Rocket<Build> {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_headers(AllowedHeaders::all())
        .allow_credentials(true)
        .to_cors();

    let mut app = rocket::build().mount("/", routes::take_orders::routes());

    if let Ok(cors) = cors {
        app = app.attach(cors);
    }

    app
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    create_app().launch().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::http::{ContentType, Status};
    use rocket::local::blocking::Client;

    fn client() -> Client {
        Client::tracked(create_app()).expect("valid rocket instance")
    }

    #[test]
    fn test_cors_headers_present() {
        let client = client();
        let response = client
            .options("/take-orders")
            .header(rocket::http::Header::new(
                "Access-Control-Request-Method",
                "POST",
            ))
            .header(rocket::http::Header::new("Origin", "http://localhost:3000"))
            .dispatch();

        assert!(response
            .headers()
            .get_one("Access-Control-Allow-Origin")
            .is_some());
    }

    #[test]
    fn test_take_orders_invalid_yaml() {
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
                "buyAmount": "100",
                "priceCap": "2.5",
                "minReceiveMode": "partial"
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
                "buyAmount": "100",
                "priceCap": "2.5",
                "minReceiveMode": "partial"
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
                "buyAmount": "100",
                "priceCap": "2.5",
                "minReceiveMode": "partial"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[test]
    fn test_take_orders_zero_buy_amount() {
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
                "buyAmount": "0",
                "priceCap": "2.5",
                "minReceiveMode": "partial"
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
                "buyAmount": "100",
                "priceCap": "-1",
                "minReceiveMode": "partial"
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
                "buyAmount": "100",
                "priceCap": "2.5",
                "minReceiveMode": "partial"
            }"#,
            )
            .dispatch();

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }
}
