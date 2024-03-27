use lazy_static::lazy_static;
use portpicker::pick_unused_port;
use test_context::AsyncTestContext;
use thirtyfour::prelude::*;
use thirtyfour::CapabilitiesHelper;
use tokio::process::{Child, Command};

const WEBDRIVER_PATH: &str = "/usr/bin/WebKitWebDriver";
const TAURI_APP_PATH: &str = "../tauri-app/src-tauri/target/release/rain-orderbook";
lazy_static! {
    static ref WEBDRIVER_PORT: u16 = pick_unused_port().expect("Failed to pick unused port");
    static ref WEBDRIVER_URL: String = format!("http://localhost:{}", *WEBDRIVER_PORT);
}

pub struct WebdriverTestContext {
    pub driver: WebDriver,
    pub driver_server: Child,
}

impl AsyncTestContext for WebdriverTestContext {
    async fn setup() -> WebdriverTestContext {
        // Launch WebKitWebDriver
        let child = Command::new(WEBDRIVER_PATH)
            .env("TAURI_AUTOMATION", "true")
            .arg(format!("--port={}", *WEBDRIVER_PORT))
            .kill_on_drop(true)
            .spawn()
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to launch WebKitWebDriver at path {} with port {}",
                    WEBDRIVER_PATH, *WEBDRIVER_PORT
                )
            });

        // Pause for WebKitWebDriver server statup delay
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Connect client and start session
        let mut capabilities = Capabilities::new();
        capabilities.add("browserName", "wry").unwrap();
        capabilities
            .add_subkey("webkitgtk:browserOptions", "binary", TAURI_APP_PATH)
            .expect("Failed to add webkitgtk:browserOptions capability");
        let driver = WebDriver::new(WEBDRIVER_URL.as_str(), capabilities)
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to start session on Webdriver server at {}",
                    *WEBDRIVER_URL
                )
            });

        // Reset app state
        let context = WebdriverTestContext {
            driver,
            driver_server: child,
        };
        context.reset_state().await;

        context
    }

    async fn teardown(mut self) {
        // End webdriver session
        self.driver.quit().await.unwrap();
        // Kill WebKitWebDriver server
        self.driver_server.kill().await.unwrap();
    }
}
