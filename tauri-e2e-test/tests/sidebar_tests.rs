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
        .expect("Failed to find sidebar-vaults element")
        .click()
        .await
        .expect("Failed to click sidebar-vaults element");
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
        .expect("Failed to find sidebar-vaults element");
    let link_href = link_elem.attr("href").await.unwrap().unwrap();

    assert!(link_href.contains("https://docs.rainlang.xyz/intro"));
}
