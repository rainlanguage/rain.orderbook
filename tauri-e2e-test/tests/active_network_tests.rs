use test_context::test_context;
use thirtyfour::prelude::*;
mod common;
use common::{
    harness::WebdriverTestContext,
    constants
};

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn open_active_network_dropdown(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::VALID_SETTINGS_MULTIPLE.to_string()).await;

    ctx.driver
        .query(By::Css("aside button:has(span[data-testid=dropdown-activenetwork])"))
        .single()
        .await
        .expect("Failed to find activenetwork dropdown button")
        .click()
        .await
        .expect("Failed to click activenetwork dropdown button");

    let options = ctx.driver
        .query(By::Css("aside div[data-testid=dropdown-activenetwork-option]"))
        .all()
        .await
        .expect("Failed to find activenetwork dropdown options");
    
    assert_eq!(options.len(), 2);
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn switch_active_network(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::VALID_SETTINGS_MULTIPLE.to_string()).await;

    // click dropdown
    ctx.driver
        .query(By::Css("aside button:has(span[data-testid=dropdown-activenetwork])"))
        .single()
        .await
        .expect("Failed to find activenetwork dropdown button")
        .click()
        .await
        .expect("Failed to click activenetwork dropdown button");

    // click first option of dropdown
    ctx.driver
        .query(By::Css("aside label:has(div[data-testid=dropdown-activenetwork-option])"))
        .first()
        .await
        .expect("Failed to find activenetwork dropdown option")
        .click()
        .await
        .expect("Failed to click activenetwork dropdown option");

    // check selected network
    let label = ctx.driver
        .query(By::Css("aside span[data-testid=dropdown-activenetwork]"))
        .single()
        .await
        .expect("Failed to find activenetwork dropdown")
        .text()
        .await
        .expect("Failed to read text from activenetwork dropdown");

    assert_eq!(label, "Ethereum");
}