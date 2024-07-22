use crate::execute::Execute;
use rain_orderbook_quote::cli::Quoter;

impl Execute for Quoter {
    async fn execute(&self) -> anyhow::Result<()> {
        self.run().await
    }
}
