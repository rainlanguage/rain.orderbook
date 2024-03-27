use test_context::test_context;
use thirtyfour::prelude::*;

mod common;
use common::WebdriverTestContext;

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn has_sidebar(ctx: &mut WebdriverTestContext) {
    let _ = ctx.driver.query(By::Css("body > aside"))
        .nowait()
        .exists()
        .await
        .unwrap();
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn navigates_to_settings_page_on_first_launch(ctx: &mut WebdriverTestContext) {
    let breadcrumb_title_elem = ctx
        .driver
        .query(By::Css("span[data-testid=breadcrumb-page-title]"))
        .single()
        .await
        .unwrap();
    let breadcrumb_title_text = breadcrumb_title_elem.text().await.unwrap();

    assert_eq!(breadcrumb_title_text, "Settings");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn navigates_to_orders_page_when_valid_network_and_orderbook_settings(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(common::MIN_VALID_SETTINGS.to_string()).await;

    let breadcrumb_title_elem = ctx
        .driver
        .query(By::Css("span[data-testid=breadcrumb-page-title]"))
        .single()
        .await
        .unwrap();
    let breadcrumb_title_text = breadcrumb_title_elem.text().await.unwrap();
    
    assert_eq!(breadcrumb_title_text, "Orders");
}