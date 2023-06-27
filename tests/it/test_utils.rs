use ethers::providers::{Http, Ipc, Provider, ProviderExt};
use eyre::Result;

/// spawns a ipc provider
pub async fn spawn_ipc_provider(ipc_path: &str) -> Result<Provider<Ipc>> {
    Ok(Provider::connect_ipc(ipc_path).await?)
}

/// spawns a http provider
pub async fn spawn_http_provider(url: &str) -> Result<Provider<Http>> {
    Ok(Provider::<Http>::connect(url).await)
}
