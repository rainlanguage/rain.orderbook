pub(crate) mod bounty;
pub(crate) mod content_meta_v1;
pub(crate) mod context_entity;
pub(crate) mod erc20;
pub(crate) mod io;
pub(crate) mod order;
pub(crate) mod order_clear;
pub(crate) mod order_clear_state_change;
pub(crate) mod orderbook;
pub(crate) mod rain_meta_v1;
pub(crate) mod take_order_entity;
pub(crate) mod token_vault;
pub(crate) mod token_vault_take_order;
pub(crate) mod vault;
pub(crate) mod vault_deposit;
pub(crate) mod vault_withdraw;

use anyhow::Result;
use ethers::types::{Address, Bytes};
use once_cell::sync::Lazy;
use reqwest::Url;

use bounty::{get_bounty, BountyResponse};
use content_meta_v1::{get_content_meta_v1, ContentMetaV1Response};
use context_entity::{get_context_entity, ContextEntityResponse};
use erc20::{get_erc20, ERC20Response};
use io::{get_i_o, IOResponse};
use order::{get_order, OrderResponse};
use order_clear::{get_order_clear, OrderClearResponse};
use order_clear_state_change::{get_order_clear_state_change, OrderClearStateChangeResponse};
use orderbook::{get_orderbook_query, OrderBookResponse};
use rain_meta_v1::{get_rain_meta_v1, RainMetaV1Response};
use take_order_entity::{get_take_order_entity, TakeOrderEntityResponse};
use token_vault::{get_token_vault, TokenVaultResponse};
use token_vault_take_order::{get_token_vault_take_order, TokenVaultTakeOrderResponse};
use vault::{get_vault, VaultResponse};
use vault_deposit::{get_vault_deposit, VaultDepositResponse};
use vault_withdraw::{get_vault_withdraw, VaultWithdrawResponse};

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

    pub async fn order_clear(id: &String) -> Result<OrderClearResponse> {
        get_order_clear(id).await
    }

    pub async fn token_vault(id: &String) -> Result<TokenVaultResponse> {
        get_token_vault(id).await
    }

    pub async fn bounty(id: &String) -> Result<BountyResponse> {
        get_bounty(id).await
    }

    pub async fn order_clear_state_change(id: &String) -> Result<OrderClearStateChangeResponse> {
        get_order_clear_state_change(id).await
    }

    pub async fn token_vault_take_order(id: &String) -> Result<TokenVaultTakeOrderResponse> {
        get_token_vault_take_order(id).await
    }

    pub async fn take_order_entity(id: &String) -> Result<TakeOrderEntityResponse> {
        get_take_order_entity(id).await
    }

    pub async fn context_entity(id: &String) -> Result<ContextEntityResponse> {
        get_context_entity(id).await
    }
}
