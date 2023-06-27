use ethers::providers::{Ipc, Provider};
use ethers_reth::{RethMiddleware, RethMiddlewareError};
use eyre::Result;
use std::path::Path;
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
) -> Result<RethMiddleware<Provider<Ipc>>, RethMiddlewareError<Provider<Ipc>>> {
    let rt = Runtime::new().unwrap();
    let handle = rt.handle().clone();
    Ok(RethMiddleware::new(provider, db_path, handle).unwrap())
}

pub async fn spawn_bench_reth_middleware(
) -> Result<RethMiddleware<Provider<Ipc>>, RethMiddlewareError<Provider<Ipc>>> {
    let rt = Runtime::new().unwrap();
    let handle = rt.handle().clone();
    let provider = Provider::connect_ipc(TEST_IPC_PATH).await.unwrap();
    let middleware = RethMiddleware::new(provider, TEST_DB_PATH.as_ref(), handle)?;
    Ok(middleware)
}
