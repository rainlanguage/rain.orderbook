use test_context::test_context;
use thirtyfour::prelude::*;
mod common;
use common::{constants, harness::WebdriverTestContext};

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_orders_page(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::MIN_VALID_SETTINGS.to_string())
        .await;

    let link_elem = ctx
        .driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-orders])"))
        .single()
        .await
        .expect("Failed to find sidebar-orders element");
    let link_href = link_elem.attr("href").await.unwrap().unwrap();

    assert_eq!(link_href, "tauri://localhost/orders");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_vaults_page(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::MIN_VALID_SETTINGS.to_string())
        .await;

    let link_elem = ctx
        .driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-vaults])"))
        .single()
        .await
        .expect("Failed to find sidebar-vaults element");
    let link_href = link_elem.attr("href").await.unwrap().unwrap();

    assert_eq!(link_href, "tauri://localhost/vaults");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_settings_page(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::MIN_VALID_SETTINGS.to_string())
        .await;

    let link_elem = ctx
        .driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-settings])"))
        .single()
        .await
        .expect("Failed to find sidebar-settings element");
    let link_href = link_elem.attr("href").await.unwrap().unwrap();

    assert_eq!(link_href, "tauri://localhost/settings");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_external_documentation(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::MIN_VALID_SETTINGS.to_string())
        .await;

    let link_elem = ctx
        .driver
        .query(By::Css(
            "aside a:has(> span[data-testid=sidebar-documentation])",
        ))
        .single()
        .await
        .expect("Failed to find sidebar-documentation element");
    let link_href = link_elem.attr("href").await.unwrap().unwrap();

    assert!(link_href.contains("https://docs.rainlang.xyz/intro"));
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_shows_active_network(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::MIN_VALID_SETTINGS.to_string())
        .await;

    let label = ctx
        .driver
        .query(By::Css("aside span[data-testid=dropdown-activenetwork]"))
        .single()
        .await
        .expect("Failed to find activenetwork dropdown")
        .text()
        .await
        .expect("Failed to read text from activenetwork dropdown");

    assert_eq!(label, "Polygon");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_shows_active_orderbook(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::MIN_VALID_SETTINGS.to_string())
        .await;

    let label = ctx
        .driver
        .query(By::Css("aside span[data-testid=dropdown-activeorderbook]"))
        .single()
        .await
        .expect("Failed to find activeorderbook dropdown")
        .text()
        .await
        .expect("Failed to read text from activeorderbook dropdown");

    assert_eq!(label, "Polygon Orderbook");
}
