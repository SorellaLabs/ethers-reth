use std::{path::Path, sync::Arc};

use ethers::providers::{Http, Provider, ProviderExt};
use ethers_reth::RethMiddleware;
use eyre::Result;
use reth_primitives::ChainSpec;
use tokio::runtime::Handle;

/// spawns a http provider
pub async fn spawn_http_provider(url: &str) -> Result<Provider<Http>> {
    Ok(Provider::<Http>::connect(url).await)
}

pub const MAINNET_HTTP_URL: &str =
    "https://eth-mainnet.g.alchemy.com/v2/mK6cfUjc2eoyoXz4t4IKgi6KDmnMZsmq";

pub async fn spawn_reth_middleware<P: AsRef<Path>>(
    http_path: &str,
    chain: Arc<ChainSpec>,
    db_path: P,
) -> RethMiddleware<Provider<Http>> {
    let http_provider =
        spawn_http_provider(http_path).await.expect("Could not spawn http provider");
    RethMiddleware::new(http_provider, db_path, Handle::current(), chain.clone())
        .expect("Could not spawn reth middleware")
}
