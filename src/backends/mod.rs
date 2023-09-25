mod cloudflare;
pub use cloudflare::*;

use crate::err::*;

use async_trait::async_trait;

#[async_trait]
pub trait DNSSync {
    async fn sync(&self, v4addr: &str) -> Result<()>;
}

#[derive(Debug)]
pub enum Backend {
    Cloudflare(Cloudflare),
}

#[async_trait]
impl DNSSync for Backend {
    async fn sync(&self, v4addr: &str) -> Result<()> {
        match self {
            Backend::Cloudflare(cloudflare) => cloudflare.sync(v4addr).await,
        }
    }
}
