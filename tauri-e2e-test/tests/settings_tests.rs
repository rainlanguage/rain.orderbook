use test_context::test_context;
use thirtyfour::prelude::*;
mod common;
use common::{
    harness::WebdriverTestContext,
    constants
};

#[test_context(WebdriverTestContext)]
#[tokio::test]
async fn type_settings_and_apply(ctx: &mut WebdriverTestContext) {
    ctx.driver.goto("tauri://localhost/settings").await.unwrap();

    ctx.driver
        .query(By::Css("div.codemirror-wrapper div[contenteditable=true]"))
        .single()
        .await
        .expect("Failed to find codemirror element")
        .send_keys(constants::MIN_VALID_SETTINGS_KEYS.as_str())
        .await
        .expect("Failed to type in codemirror element");
        
    ctx.driver
        .query(By::Css("button:has(span[data-testid=button-applysettings])"))
        .and_clickable()
        .single()
        .await
        .expect("Failed to find applysettings button")
        .click()
        .await
        .expect("Failed to click applysettings button");

    let settings_stored = ctx.read_localstorage("settings".to_string()).await;

    assert_eq!(settings_stored, constants::MIN_VALID_SETTINGS.to_string());
}
