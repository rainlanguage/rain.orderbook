use test_context::test_context;
use thirtyfour::prelude::*;
mod common;
use common::{constants, harness::WebdriverTestContext, utils::sleep_ms};

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn open_active_orderbook_dropdown(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::VALID_SETTINGS_MULTIPLE.to_string())
        .await;

    ctx.driver.goto("tauri://localhost/orders").await.unwrap();

    ctx.write_localstorage(
        "settings.activeNetworkRef".to_string(),
        "polygon".to_string(),
    )
    .await;
    ctx.driver.refresh().await.unwrap();

    ctx.driver
        .query(By::Css(
            "aside button:has(span[data-testid=dropdown-activeorderbook])",
        ))
        .single()
        .await
        .expect("Failed to find activeorderbook dropdown button")
        .click()
        .await
        .expect("Failed to click activeorderbook dropdown button");

    let options = ctx
        .driver
        .query(By::Css(
            "aside div[data-testid=dropdown-activeorderbook-option]",
        ))
        .all()
        .await
        .expect("Failed to find activeorderbook dropdown options");

    assert_eq!(options.len(), 2);
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn switch_active_network_changes_available_orderbooks(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::VALID_SETTINGS_MULTIPLE.to_string())
        .await;

    ctx.driver.goto("tauri://localhost/orders").await.unwrap();

    // click dropdown
    ctx.driver
        .query(By::Css(
            "aside button:has(span[data-testid=dropdown-activenetwork])",
        ))
        .single()
        .await
        .expect("Failed to find activenetwork dropdown button")
        .click()
        .await
        .expect("Failed to click activenetwork dropdown button");

    // click last option of dropdown
    ctx.driver
        .query(By::Css(
            "aside label:has(div[data-testid=dropdown-activenetwork-option])",
        ))
        .all()
        .await
        .expect("Failed to find activenetwork dropdown option")
        .last()
        .unwrap()
        .click()
        .await
        .expect("Failed to click last activenetwork dropdown option");
    sleep_ms(1000).await;

    // check selected orderbook
    let label = ctx
        .driver
        .query(By::Css("aside span[data-testid=dropdown-activeorderbook]"))
        .single()
        .await
        .expect("Failed to find activeorderbook dropdown")
        .text()
        .await
        .expect("Failed to read text from activeorderbook dropdown");

    assert!(label.contains("Polygon Orderbook"));
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn switch_active_orderbook(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::VALID_SETTINGS_MULTIPLE.to_string())
        .await;
    ctx.write_localstorage(
        "settings.activeNetworkRef".to_string(),
        "polygon".to_string(),
    )
    .await;
    ctx.driver.refresh().await.unwrap();

    // click dropdown
    ctx.driver
        .query(By::Css(
            "aside button:has(span[data-testid=dropdown-activeorderbook])",
        ))
        .single()
        .await
        .expect("Failed to find activeorderbook dropdown button")
        .click()
        .await
        .expect("Failed to click activeorderbook dropdown button");

    // click last option of dropdown
    ctx.driver
        .query(By::Css(
            "aside label:has(div[data-testid=dropdown-activeorderbook-option])",
        ))
        .all()
        .await
        .expect("Failed to find last activeorderbook dropdown option")
        .last()
        .unwrap()
        .click()
        .await
        .expect("Failed to click activeorderbook dropdown option");

    // check selected orderbook
    let label = ctx
        .driver
        .query(By::Css("aside span[data-testid=dropdown-activeorderbook]"))
        .single()
        .await
        .expect("Failed to find activeorderbook dropdown")
        .text()
        .await
        .expect("Failed to read text from activeorderbook dropdown");

    assert_eq!(label, "Polygon Orderbook 2");
}
