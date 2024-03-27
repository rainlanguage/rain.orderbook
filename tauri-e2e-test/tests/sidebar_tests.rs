use test_context::test_context;
use thirtyfour::prelude::*;

mod common;
use common::WebdriverTestContext;

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_orders_page(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(common::MIN_VALID_SETTINGS.to_string()).await;

    ctx.driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-orders])"))
        .single()
        .await
        .expect("Failed to find sidebar-orders element")
        .click()
        .await
        .expect("Failed to click sidebar-orders element");

    let page_title = ctx.read_page_title().await;

    assert_eq!(page_title, "Orders");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_vaults_page(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(common::MIN_VALID_SETTINGS.to_string()).await;

    ctx.driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-vaults])"))
        .single()
        .await
        .expect("Failed to find sidebar-vaults element")
        .click()
        .await
        .expect("Failed to click sidebar-vaults element");

    let page_title = ctx.read_page_title().await;

    assert_eq!(page_title, "Vaults");
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_settings_page(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(common::MIN_VALID_SETTINGS.to_string()).await;

    ctx.driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-vaults])"))
        .single()
        .await
        .expect("Failed to find sidebar-vaults element")
        .click()
        .await
        .expect("Failed to click sidebar-vaults element");
    let page_title = ctx.read_page_title().await;

    assert_eq!(page_title, "Vaults");
    println!("visited vaults page");

    ctx.driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-settings])"))
        .single()
        .await
        .expect("Failed to find sidebar-settings element")
        .click()
        .await
        .expect("Failed to click sidebar-settings element");
    let page_title = ctx.read_page_title().await;

    assert_eq!(page_title, "Settings");
    println!("visited settings page");

}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_link_to_external_documentation(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(common::MIN_VALID_SETTINGS.to_string()).await;

    let link_elem = ctx.driver
        .query(By::Css("aside a:has(> span[data-testid=sidebar-documentation])"))
        .single()
        .await
        .expect("Failed to find sidebar-documentation element");
    let link_href = link_elem.attr("href").await.unwrap().unwrap();

    assert!(link_href.contains("https://docs.rainlang.xyz/intro"));
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn sidebar_shows_active_network(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(common::MIN_VALID_SETTINGS.to_string()).await;

    let label = ctx.driver
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
    ctx.apply_settings(common::MIN_VALID_SETTINGS.to_string()).await;

    let label = ctx.driver
        .query(By::Css("aside span[data-testid=dropdown-activeorderbook]"))
        .single()
        .await
        .expect("Failed to find activeorderbook dropdown")
        .text()
        .await
        .expect("Failed to read text from activeorderbook dropdown");

    assert_eq!(label, "Polygon Orderbook");
}
