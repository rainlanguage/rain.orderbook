use anyhow::Result;

pub trait Execute {
    fn execute(&self) -> impl std::future::Future<Output = Result<()>>;
}
