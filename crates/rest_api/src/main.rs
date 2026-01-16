mod error;
mod routes;

use rocket::http::Method;
use rocket::{launch, Build, Rocket};
use rocket_cors::{AllowedHeaders, AllowedOrigins, CorsOptions};

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
}
