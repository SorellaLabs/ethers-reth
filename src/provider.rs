use ethers::providers::{Http, Ipc, Provider, ProviderExt};
use reth_db::{mdbx::{WriteMap, Env}, database::{DatabaseGAT, Database}, DatabaseError, transaction::DbTx};

/// spawns a ipc provider
pub async fn spawn_ipc_provider(ipc_path: &str) -> Provider<Ipc> {
    Provider::connect_ipc(ipc_path).await.unwrap()
}

/// spawns a http provider
pub async fn spawn_http_provider(url: &str) -> Provider<Http> {
    Provider::connect(url).await
}

/// re-implementation of 'view()'
/// allows for a function to be passed in through a RO libmdbx transaction
/// /reth/crates/storage/db/src/abstraction/database.rs
pub fn view<F, T>(db: &Env<WriteMap>, f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&<Env<WriteMap> as DatabaseGAT<'_>>::TX) -> T,
{
    let tx = db.tx().unwrap();
    let res = f(&tx);
    tx.commit().unwrap();

    Ok(res)
}