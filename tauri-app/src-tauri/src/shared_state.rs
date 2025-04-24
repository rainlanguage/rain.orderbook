use crate::error::CommandResult;
use rain_orderbook_common::fuzz::FuzzRunner;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct SharedState {
    fuzz_runner: Mutex<HashMap<(String, Option<String>), Arc<FuzzRunner>>>,
}

impl SharedState {
    pub async fn get_or_create_fuzz_runner(
        &self,
        dotrain: String,
        settings: Option<String>,
    ) -> CommandResult<Arc<FuzzRunner>> {
        let key = (dotrain, settings);
        let mut cache = self.fuzz_runner.lock().await;

        if let Some(runner) = cache.get(&key) {
            Ok(runner.clone())
        } else {
            let runner = Arc::new(FuzzRunner::new(&key.0, key.1.clone(), None).await?);
            cache.insert(key, runner.clone());
            Ok(runner)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    const DOTRAIN1: &str = r#"
networks:
  mainnet:
    rpc_url: "https://mainnet.infura.io/v3/YOUR_INFURA_API_KEY"
    chain_id: 1
---
#calculate-io
_ _: 1 2;
#handle-io
:;
#handle-add-order
"#;
    const DOTRAIN2: &str = r#"
networks:
  mainnet:
    rpc_url: https://test.com
    chain_id: 1
subgraphs:
  mainnet:
    url: https://test.com
---
#calculate-io
_ _: 1 2;
#handle-io
:;
#handle-add-order
"#;

    #[tokio::test]
    async fn test_shared_state_fuzz_runner_cache_hit() {
        let shared_state = SharedState::default();

        let runner1 = shared_state
            .get_or_create_fuzz_runner(DOTRAIN1.to_string(), None)
            .await
            .expect("First creation failed");

        let runner2 = shared_state
            .get_or_create_fuzz_runner(DOTRAIN1.to_string(), None)
            .await
            .expect("Second call failed");

        assert!(
            Arc::ptr_eq(&runner1, &runner2),
            "Cache miss: Runners should be the same instance"
        );
    }

    #[tokio::test]
    async fn test_shared_state_fuzz_runner_cache_miss_different_script() {
        let shared_state = SharedState::default();

        let runner1 = shared_state
            .get_or_create_fuzz_runner(DOTRAIN1.to_string(), None)
            .await
            .expect("Runner 1 creation failed");
        let runner2 = shared_state
            .get_or_create_fuzz_runner(DOTRAIN2.to_string(), None)
            .await
            .expect("Runner 2 creation failed");

        assert!(
            !Arc::ptr_eq(&runner1, &runner2),
            "Cache hit: Different scripts should produce different runners"
        );
    }

    #[tokio::test]
    async fn test_shared_state_fuzz_runner_cache_miss_different_settings() {
        let shared_state = SharedState::default();
        let settings1 = Some("settings A".to_string());
        let settings2 = Some("settings B".to_string());

        let runner_none = shared_state
            .get_or_create_fuzz_runner(DOTRAIN1.to_string(), None)
            .await
            .expect("Runner None creation failed");
        let runner_s1 = shared_state
            .get_or_create_fuzz_runner(DOTRAIN1.to_string(), settings1.clone())
            .await
            .expect("Runner S1 creation failed");
        let runner_s2 = shared_state
            .get_or_create_fuzz_runner(DOTRAIN1.to_string(), settings2.clone())
            .await
            .expect("Runner S2 creation failed");
        let runner_s1_again = shared_state
            .get_or_create_fuzz_runner(DOTRAIN1.to_string(), settings1.clone())
            .await
            .expect("Runner S1 again creation failed");

        assert!(
            !Arc::ptr_eq(&runner_none, &runner_s1),
            "Cache hit: None vs Some settings"
        );
        assert!(
            !Arc::ptr_eq(&runner_s1, &runner_s2),
            "Cache hit: Different Some settings"
        );
        assert!(
            Arc::ptr_eq(&runner_s1, &runner_s1_again),
            "Cache miss: Same Some settings should hit cache"
        );
    }
}
