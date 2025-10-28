use rain_orderbook_common::local_db::{
    pipeline::{
        adapters::bootstrap::DefaultBootstrapAdapter,
        traits::{BootstrapConfig, BootstrapPipeline, BootstrapState, TargetKey},
    },
    query::LocalDbQueryExecutor,
    LocalDbError,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ProducerBootstrapAdapter;

#[async_trait::async_trait(?Send)]
impl BootstrapPipeline for ProducerBootstrapAdapter {
    async fn ensure_schema<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        DefaultBootstrapAdapter::new()
            .ensure_schema(db, db_schema_version)
            .await
    }

    async fn inspect_state<DB>(
        &self,
        db: &DB,
        target_key: &TargetKey,
    ) -> Result<BootstrapState, LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        DefaultBootstrapAdapter::new()
            .inspect_state(db, target_key)
            .await
    }

    async fn reset_db<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        DefaultBootstrapAdapter::new()
            .reset_db(db, db_schema_version)
            .await
    }

    async fn run<DB>(
        &self,
        db: &DB,
        db_schema_version: Option<u32>,
        config: &BootstrapConfig,
    ) -> Result<(), LocalDbError>
    where
        DB: LocalDbQueryExecutor + ?Sized,
    {
        self.reset_db(db, db_schema_version).await?;

        if let Some(dump_stmt) = &config.dump_stmt {
            db.query_text(dump_stmt).await?;
        }

        Ok(())
    }
}
