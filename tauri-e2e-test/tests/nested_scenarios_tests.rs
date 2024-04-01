use test_context::test_context;
use thirtyfour::prelude::*;
mod common;
use common::{constants, harness::WebdriverTestContext};

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn open_scenarios_dropdown(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::VALID_WITH_NESTED_SCENARIO.to_string())
        .await;

    ctx.goto_add_order().await;

    ctx.driver
        .query(By::Css(
            "aside button:has(span[data-testid=dropdown-activescenario])",
        ))
        .single()
        .await
        .expect("Failed to find scenarios dropdown button")
        .click()
        .await
        .expect("Failed to click scenarios dropdown button");

    let options = ctx
        .driver
        .query(By::Css("aside div[data-testid=dropdown-scenarios-option]"))
        .all()
        .await
        .expect("Failed to find scenarios dropdown options");

    assert_eq!(options.len(), 3);
}

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn switch_scenario(ctx: &mut WebdriverTestContext) {
    ctx.apply_settings(constants::VALID_WITH_NESTED_SCENARIO.to_string())
        .await;

    ctx.goto_add_order().await;

    // click dropdown
    ctx.driver
        .query(By::Css(
            "aside button:has(span[data-testid=dropdown-activescenario])",
        ))
        .single()
        .await
        .expect("Failed to find scenarios dropdown button")
        .click()
        .await
        .expect("Failed to click scenarios dropdown button");

    // click last option of dropdown
    ctx.driver
        .query(By::Css(
            "aside label:has(div[data-testid=dropdown-scenarios-option])",
        ))
        .all()
        .await
        .expect("Failed to find scenarios dropdown option")[2]
        .click()
        .await
        .expect("Failed to click scenarios dropdown option");

    // check selected scenario
    let label = ctx
        .driver
        .query(By::Css("aside span[data-testid=dropdown-activescenario]"))
        .single()
        .await
        .expect("Failed to find scenarios dropdown")
        .text()
        .await
        .expect("Failed to read text from scenarios dropdown");

    assert_eq!(label, "polygon.sell");
}
