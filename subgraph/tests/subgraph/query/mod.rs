pub(crate) mod content_meta_v1;
pub(crate) mod io;
pub(crate) mod order;
pub(crate) mod orderbook;
pub(crate) mod rain_meta_v1;
pub(crate) mod vault;
pub(crate) mod vault_deposit;
pub(crate) mod vault_withdraw;
pub(crate) mod erc20;

use anyhow::Result;
use ethers::types::{Address, Bytes};
use once_cell::sync::Lazy;
use reqwest::Url;

use content_meta_v1::{get_content_meta_v1, ContentMetaV1Response};
use io::{get_i_o, IOResponse};
use order::{get_order, OrderResponse};
use orderbook::{get_orderbook_query, OrderBookResponse};
use rain_meta_v1::{get_rain_meta_v1, RainMetaV1Response};
use vault::{get_vault, VaultResponse};
use vault_deposit::{get_vault_deposit, VaultDepositResponse};
use vault_withdraw::{get_vault_withdraw, VaultWithdrawResponse};
use erc20::{get_erc20, ERC20Response};

pub static SG_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("http://localhost:8000/subgraphs/name/test/test").unwrap());

pub struct Query;

impl Query {
    pub async fn orderbook(id: &Address) -> Result<OrderBookResponse> {
        get_orderbook_query(id).await
    }

    pub async fn rain_meta_v1(id: &Bytes) -> Result<RainMetaV1Response> {
        get_rain_meta_v1(id).await
    }

    pub async fn content_meta_v1(id: &Bytes) -> Result<ContentMetaV1Response> {
        get_content_meta_v1(id).await
    }

    pub async fn order(id: &Bytes) -> Result<OrderResponse> {
        get_order(id).await
    }

    pub async fn i_o(id: &String) -> Result<IOResponse> {
        get_i_o(id).await
    }

    pub async fn vault(id: &String) -> Result<VaultResponse> {
        get_vault(id).await
    }

    pub async fn vault_deposit(id: &String) -> Result<VaultDepositResponse> {
        get_vault_deposit(id).await
    }

    pub async fn vault_withdraw(id: &String) -> Result<VaultWithdrawResponse> {
        get_vault_withdraw(id).await
    }

    pub async fn erc20(id: &Address) -> Result<ERC20Response> {
        get_erc20(id).await
    }
}
