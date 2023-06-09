use ethers::providers::{Http, Ipc, Provider, ProviderExt};
use reth_db::{mdbx::{WriteMap, Env}, database::{DatabaseGAT, Database}, DatabaseError, transaction::DbTx};
use eyre::Result;

/// spawns a ipc provider
pub async fn spawn_ipc_provider(ipc_path: &str) -> Result<Provider<Ipc>> {
    Ok(Provider::connect_ipc(ipc_path).await?)
}

/// spawns a http provider
pub async fn spawn_http_provider(url: &str) -> Result<Provider<Http>> {
    Ok(Provider::connect(url).await)
}

/// re-implementation of 'view()'
/// allows for a function to be passed in through a RO libmdbx transaction
/// /reth/crates/storage/db/src/abstraction/database.rs
pub fn view<F, T>(db: &Env<WriteMap>, f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&<Env<WriteMap> as DatabaseGAT<'_>>::TX) -> T,
{
    let tx = db.tx()?;
    let res = f(&tx);
    tx.commit()?;

    Ok(res)
}