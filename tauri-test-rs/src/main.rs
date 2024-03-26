use thirtyfour::prelude::*;
use thirtyfour::CapabilitiesHelper;

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    let mut capabilities = Capabilities::new();
    capabilities.add("browserName", "wry").unwrap();
    capabilities
        .add_subkey(
            "webkitgtk:browserOptions",
            "binary",
            "tauri-app/src-tauri/target/release/rain-orderbook",
        )
        .unwrap();

    let driver = WebDriver::new("http://localhost:4444", capabilities).await?;

    // Navigate to https://wikipedia.org.
    driver.goto("https://wikipedia.org").await?;
    let elem_form = driver.find(By::Id("search-form")).await?;

    // Always explicitly close the browser.
    driver.quit().await?;

    Ok(())
}
