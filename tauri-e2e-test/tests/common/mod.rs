use tokio::{
    process::{Child, Command},
    time::sleep
};
use std::time::Duration;
use test_context::AsyncTestContext;
use thirtyfour::prelude::*;
use thirtyfour::CapabilitiesHelper;
use serde_json::Value;
use portpicker::pick_unused_port;
use lazy_static::lazy_static;

mod settings;
pub use settings::*;

const WEBDRIVER_PATH: &str = "WebKitWebDriver";
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
            .expect(
                format!(
                    "Failed to launch WebKitWebDriver at path {} with port {}",
                    WEBDRIVER_PATH,
                    *WEBDRIVER_PORT
                )
                .as_str(),
            );
            
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
            .expect(format!("Failed to start session on Webdriver server at {}", *WEBDRIVER_URL).as_str());

        // Reset app state
        let context = WebdriverTestContext {
            driver,
            driver_server: child
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


impl WebdriverTestContext {
    /// Wait for svelte app to load
    async fn app_exists(&self) {
        self.driver.query(By::Css("main")).exists().await.unwrap();
    }

    /// Navigate to home and refresh
    async fn goto_home(&self) {
        self.driver.goto("tauri://localhost/").await.unwrap();
        self.driver.refresh().await.unwrap();
        Self::sleep_500ms().await;
    }
    
    /// Reset app state
    pub async fn reset_state(&self) {
        self.driver.execute(r#"
            localStorage.clear();
        "#, vec![]).await.unwrap();
        self.goto_home().await;
        self.app_exists().await;
    }

    /// Apply settings
    pub async fn apply_settings(&self, settings: String) {
        self.driver.execute(r#"
            localStorage.setItem("settings", arguments[0]);
        "#, vec![Value::String(settings)]).await.unwrap();
        self.goto_home().await;
        self.app_exists().await;
    }

    /// Read breadcrumb title
    pub async fn read_page_title(&self) -> String {
        let breadcrumb_title_elem = self
            .driver
            .query(By::Css("span[data-testid=breadcrumb-page-title]"))
            .single()
            .await
            .unwrap();
        
        breadcrumb_title_elem.text().await.unwrap()
    }

    pub async fn sleep_500ms() {
        sleep(Duration::from_millis(500)).await;
    }
}