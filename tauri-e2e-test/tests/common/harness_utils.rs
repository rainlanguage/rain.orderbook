#![allow(dead_code)]

use crate::common::harness::WebdriverTestContext;
use crate::common::utils::sleep_ms;
use serde_json::Value;
use thirtyfour::prelude::*;

impl WebdriverTestContext {
    /// Wait for svelte app to load
    async fn app_exists(&self) {
        self.driver.query(By::Css("main")).exists().await.unwrap();
    }

    /// Navigate to home and refresh
    async fn goto_home(&self) {
        self.driver.goto("tauri://localhost/").await.unwrap();
        self.driver.refresh().await.unwrap();
        sleep_ms(500).await;
    }

    /// Navigate to add order page
    pub async fn goto_add_order(&self) {
        self.driver
            .goto("tauri://localhost/orders/add/")
            .await
            .unwrap();
        sleep_ms(500).await;
    }

    /// Reset app state
    pub async fn reset_state(&self) {
        self.driver
            .execute(
                r#"
          localStorage.clear();
      "#,
                vec![],
            )
            .await
            .unwrap();
        self.goto_home().await;
        self.app_exists().await;
    }

    /// Read value from localstorage
    pub async fn read_localstorage(&self, key: String) -> String {
        let res = self
            .driver
            .execute(
                r#"
              return localStorage.getItem(arguments[0]);
          "#,
                vec![Value::String(key.clone())],
            )
            .await
            .unwrap();

        if let Value::String(value_string) = res.json() {
            value_string.to_string()
        } else {
            panic!("Failed to read localstorage key {}", key);
        }
    }

    /// Write value to localstorage
    pub async fn write_localstorage(&self, key: String, value: String) {
        self.driver
            .execute(
                r#"
              localStorage.setItem(arguments[0], arguments[1]);
          "#,
                vec![Value::String(key.clone()), Value::String(value.clone())],
            )
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to write localstorage key {} to value {}",
                    key, value
                )
            });
    }

    /// Apply settings
    pub async fn apply_settings(&self, settings: String) {
        self.driver
            .execute(
                r#"
          localStorage.setItem("settings", arguments[0]);
      "#,
                vec![Value::String(settings)],
            )
            .await
            .unwrap();
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
}
