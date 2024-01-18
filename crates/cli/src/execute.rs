use anyhow::Result;

pub trait Execute {
    async fn execute(&self) -> Result<()>;
}
