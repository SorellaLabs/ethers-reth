use ethers::providers::{Ipc, Provider};
use ethers_reth::{RethMiddleware, RethMiddlewareError};
use eyre::Result;
use reth_primitives::{ChainSpec, MAINNET};
use std::{path::Path, sync::Arc};
use tokio::runtime::Runtime;

const TEST_IPC_PATH: &str = "/tmp/reth.ipc";
const TEST_DB_PATH: &str = "/NVMe/data/reth/db";

pub async fn spawn_ipc_provider(
    ipc_path: &str,
) -> Result<Provider<Ipc>, ethers::providers::ProviderError> {
    Provider::connect_ipc(ipc_path).await
}

pub async fn spawn_bench_ipc_provider() -> Result<Provider<Ipc>, ethers::providers::ProviderError> {
    Provider::connect_ipc(TEST_IPC_PATH).await
}

pub async fn spawn_reth_middleware(
    provider: Provider<Ipc>,
    db_path: &Path,
    chain: Option<Arc<ChainSpec>>,
) -> Result<RethMiddleware<Provider<Ipc>>, RethMiddlewareError<Provider<Ipc>>> {
    let rt = Runtime::new().unwrap();
    let handle = rt.handle().clone();
    Ok(RethMiddleware::new(
        provider,
        db_path,
        handle,
        chain.unwrap_or(MAINNET.clone()).chain().id(),
    )
    .unwrap())
}

pub async fn spawn_bench_reth_middleware(
) -> Result<RethMiddleware<Provider<Ipc>>, RethMiddlewareError<Provider<Ipc>>> {
    let _rt = Runtime::new().unwrap();
    let provider = Provider::connect_ipc(TEST_IPC_PATH).await.unwrap();
    let middleware =
        spawn_reth_middleware(provider, Path::new(TEST_DB_PATH), Some(MAINNET.clone())).await?;
    Ok(middleware)
}
