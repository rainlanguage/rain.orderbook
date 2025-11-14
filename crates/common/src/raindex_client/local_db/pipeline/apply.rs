use crate::erc20::TokenInfo;
use crate::local_db::decode::{DecodedEvent, DecodedEventData};
use crate::local_db::pipeline::adapters::apply::DefaultApplyPipeline;
use crate::local_db::pipeline::{ApplyPipeline, ApplyPipelineTargetInfo};
use crate::local_db::query::fetch_erc20_tokens_by_addresses::Erc20TokenRow;
use crate::local_db::query::upsert_materialized_vault_balances::upsert_materialized_vault_balances_stmt;
use crate::local_db::query::{LocalDbQueryExecutor, SqlStatementBatch};
use crate::local_db::{LocalDbError, OrderbookIdentifier};
use crate::rpc_client::LogEntryResponse;
use alloy::primitives::Address;
use async_trait::async_trait;

/// Client-specific Apply adapter that augments the default pipeline with
/// post-batch statements (materialized vault balances refresh).
#[derive(Debug, Clone, Default)]
pub struct ClientApplyAdapter {
    inner: DefaultApplyPipeline,
}

impl ClientApplyAdapter {
    pub const fn new() -> Self {
        Self {
            inner: DefaultApplyPipeline::new(),
        }
    }

    fn build_materialized_batch(&self, target_info: &ApplyPipelineTargetInfo) -> SqlStatementBatch {
        if target_info.start_block > target_info.target_block {
            return SqlStatementBatch::new();
        }

        SqlStatementBatch::from(vec![upsert_materialized_vault_balances_stmt(
            &target_info.ob_id,
            target_info.start_block,
            target_info.target_block,
        )])
    }
}

#[async_trait(?Send)]
impl ApplyPipeline for ClientApplyAdapter {
    fn build_batch(
        &self,
        target_info: &ApplyPipelineTargetInfo,
        raw_logs: &[LogEntryResponse],
        decoded_events: &[DecodedEventData<DecodedEvent>],
        existing_tokens: &[Erc20TokenRow],
        tokens_to_upsert: &[(Address, TokenInfo)],
    ) -> Result<SqlStatementBatch, LocalDbError> {
        self.inner.build_batch(
            target_info,
            raw_logs,
            decoded_events,
            existing_tokens,
            tokens_to_upsert,
        )
    }

    fn build_post_batch(
        &self,
        target_info: &ApplyPipelineTargetInfo,
    ) -> Result<SqlStatementBatch, LocalDbError> {
        Ok(self.build_materialized_batch(target_info))
    }

    async fn persist<DB>(&self, db: &DB, batch: &SqlStatementBatch) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        self.inner.persist(db, batch).await
    }

    async fn export_dump<DB>(
        &self,
        db: &DB,
        ob_id: &OrderbookIdentifier,
        end_block: u64,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        self.inner.export_dump(db, ob_id, end_block).await
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
    fn build_post_batch_includes_materialized_upsert() {
        let adapter = ClientApplyAdapter::new();
        let batch = adapter
            .build_post_batch(&target(5, 10))
            .expect("post batch ok");
        assert!(!batch.is_empty());
        let statements = batch.statements();
        assert_eq!(statements.len(), 1, "single upsert statement expected");
        let stmt = &statements[0];
        assert!(
            stmt.sql().contains("materialized_vault_balances"),
            "refresh statement should target materialized_vault_balances"
        );
        assert_eq!(
            stmt.params(),
            &[
                SqlValue::U64(137),
                SqlValue::Text("0x00000000000000000000000000000000000000aa".into()),
                SqlValue::U64(5),
                SqlValue::U64(10)
            ]
        );
    }

    #[test]
    fn build_post_batch_empty_for_invalid_window() {
        let adapter = ClientApplyAdapter::new();
        let batch = adapter
            .build_post_batch(&target(100, 50))
            .expect("post batch ok");
        assert!(batch.is_empty());
    }
}
