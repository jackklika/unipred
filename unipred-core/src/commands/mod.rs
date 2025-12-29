pub mod quote;
pub mod markets;

use async_trait::async_trait;
use crate::UnipredCore;

#[async_trait]
pub trait Command {
    type Response;
    async fn execute(&self, core: &UnipredCore) -> anyhow::Result<Self::Response>;
}
