use crate::local_db::pipeline::adapters::apply::{ApplyPipeline, ApplyPipelineTargetInfo};
use crate::local_db::query::upsert_vault_balances::upsert_vault_balances_batch;
use crate::local_db::query::SqlStatementBatch;
use crate::local_db::LocalDbError;
use async_trait::async_trait;

/// Client-specific Apply adapter that augments the default pipeline with
/// post-batch statements (running vault balances refresh).
#[derive(Debug, Clone, Default)]
pub struct ClientApplyAdapter;

#[async_trait(?Send)]
impl ApplyPipeline for ClientApplyAdapter {
    fn build_post_batch(
        &self,
        target_info: &ApplyPipelineTargetInfo,
    ) -> Result<SqlStatementBatch, LocalDbError> {
        if target_info.start_block > target_info.target_block {
            return Ok(SqlStatementBatch::new());
        }

        Ok(upsert_vault_balances_batch(
            &target_info.ob_id,
            target_info.start_block,
            target_info.target_block,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_db::query::SqlValue;
    use crate::local_db::OrderbookIdentifier;
    use alloy::primitives::{address, b256};

    fn target(start: u64, end: u64) -> ApplyPipelineTargetInfo {
        ApplyPipelineTargetInfo {
            ob_id: OrderbookIdentifier::new(
                137,
                address!("00000000000000000000000000000000000000aa"),
            ),
            start_block: start,
            target_block: end,
            hash: b256!("0x1111111111111111111111111111111111111111111111111111111111111111"),
        }
    }

    #[test]
    fn build_post_batch_includes_running_balance_upsert() {
        let adapter = ClientApplyAdapter;
        let target_info = target(5, 10);
        let batch = adapter
            .build_post_batch(&target_info)
            .expect("post batch ok");
        assert_eq!(
            batch.len(),
            2,
            "expected change-log and running-balance statements"
        );
        for stmt in batch.statements() {
            assert_eq!(
                stmt.params(),
                &[
                    SqlValue::U64(target_info.ob_id.chain_id as u64),
                    SqlValue::Text(
                        target_info
                            .ob_id
                            .orderbook_address
                            .to_string()
                            .to_lowercase()
                    ),
                    SqlValue::U64(target_info.start_block),
                    SqlValue::U64(target_info.target_block)
                ]
            );
        }
        assert!(batch.statements()[0]
            .sql()
            .contains("vault_balance_changes"));
        assert!(batch.statements()[1]
            .sql()
            .contains("running_vault_balances"));
    }

    #[test]
    fn build_post_batch_empty_for_invalid_window() {
        let adapter = ClientApplyAdapter;
        let batch = adapter
            .build_post_batch(&target(100, 50))
            .expect("post batch ok");
        assert!(batch.is_empty());
    }
}
