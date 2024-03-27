use test_context::test_context;
use thirtyfour::prelude::*;
mod common;
use common::{constants, harness::WebdriverTestContext};

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn has_sidebar(ctx: &mut WebdriverTestContext) {
    let _ = ctx
        .driver
        .query(By::Css("aside"))
        .nowait()
        .exists()
        .await
        .unwrap();
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn navigates_to_settings_page_on_first_launch(ctx: &mut WebdriverTestContext) {
    let page_title = ctx.read_page_title().await;

    assert_eq!(page_title, "Settings");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn navigates_to_orders_page_when_valid_network_and_orderbook_settings(
    ctx: &mut WebdriverTestContext,
) {
    ctx.apply_settings(constants::MIN_VALID_SETTINGS.to_string())
        .await;

    let page_title = ctx.read_page_title().await;

    assert_eq!(page_title, "Orders");
}
