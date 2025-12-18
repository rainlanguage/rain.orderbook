use rain_orderbook_common::raindex_client::RaindexError;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use rocket::Request;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorResponse {
    pub error: String,
    pub readable_message: String,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Raindex(#[from] RaindexError),
}

impl ApiError {
    fn status_code(&self) -> Status {
        match self {
            ApiError::Raindex(e) => match e {
                RaindexError::InvalidYamlConfig
                | RaindexError::YamlError(_)
                | RaindexError::FromHexError(_)
                | RaindexError::U256ParseError(_)
                | RaindexError::I256ParseError(_)
                | RaindexError::ZeroAmount
                | RaindexError::NegativeAmount
                | RaindexError::NonPositiveBuyAmount
                | RaindexError::NonPositivePriceCap
                | RaindexError::NegativePriceCap
                | RaindexError::SameTokenPair
                | RaindexError::Float(_)
                | RaindexError::ParseInt(_) => Status::BadRequest,

                RaindexError::NoLiquidity | RaindexError::InsufficientLiquidity { .. } => {
                    Status::NotFound
                }

                RaindexError::ChainIdNotFound(_)
                | RaindexError::OrderbookNotFound(_, _)
                | RaindexError::OrderNotFound(_, _, _)
                | RaindexError::VaultNotFound(_, _, _)
                | RaindexError::SubgraphNotFound(_, _)
                | RaindexError::SubgraphNotConfigured(_)
                | RaindexError::NoNetworksConfigured => Status::NotFound,

                _ => Status::InternalServerError,
            },
        }
    }

    fn to_response(&self) -> ApiErrorResponse {
        match self {
            ApiError::Raindex(e) => ApiErrorResponse {
                error: e.to_string(),
                readable_message: e.to_readable_msg(),
            },
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status_code();
        let body = self.to_response();

        response::Response::build_from(Json(body).respond_to(request)?)
            .status(status)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bad_request_errors() {
        let err = ApiError::Raindex(RaindexError::NonPositiveBuyAmount);
        assert_eq!(err.status_code(), Status::BadRequest);

        let err = ApiError::Raindex(RaindexError::SameTokenPair);
        assert_eq!(err.status_code(), Status::BadRequest);
    }

    #[test]
    fn test_not_found_errors() {
        let err = ApiError::Raindex(RaindexError::NoLiquidity);
        assert_eq!(err.status_code(), Status::NotFound);

        let err = ApiError::Raindex(RaindexError::ChainIdNotFound(999));
        assert_eq!(err.status_code(), Status::NotFound);
    }

    #[test]
    fn test_error_response_format() {
        let err = ApiError::Raindex(RaindexError::NonPositiveBuyAmount);
        let response = err.to_response();

        assert!(!response.error.is_empty());
        assert!(!response.readable_message.is_empty());
    }
}
