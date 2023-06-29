use crate::{RethApi, RethClient, RethDebug, RethFilter, RethTrace, RethTxPool};
use eyre::Context;
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};

use reth_db::{
    database::{Database, DatabaseGAT},
    mdbx::{Env, WriteMap},
    tables,
    transaction::DbTx,
    DatabaseError,
};

use reth_network_api::test_utils::NoopNetwork;
use reth_primitives::MAINNET;
use reth_provider::{providers::BlockchainProvider, ProviderFactory};
use reth_revm::Factory;
use reth_rpc::{
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    DebugApi, EthApi, EthFilter, TraceApi, TracingCallGuard,
};
use reth_tasks::{TaskManager, TaskSpawner};
use reth_transaction_pool::EthTransactionValidator;
use std::{fmt::Debug, path::Path, sync::Arc};

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

/// Opens up an existing database at the specified path.
pub fn init_db<P: AsRef<Path> + Debug>(path: P) -> eyre::Result<Env<WriteMap>> {
    std::fs::create_dir_all(path.as_ref())?;
    println!("{:?}", path);
    let db = reth_db::mdbx::Env::<reth_db::mdbx::WriteMap>::open(
        path.as_ref(),
        reth_db::mdbx::EnvKind::RO,
    )?;

    view(&db, |tx| {
        for table in tables::Tables::ALL.iter().map(|table| table.name()) {
            tx.inner.open_db(Some(table)).wrap_err("Could not open db.").unwrap();
        }
    })?;

    Ok(db)
}

// EthApi/Filter Client
pub fn init_client(db_path: &Path) -> Result<RethClient, DatabaseError> {
    let chain = MAINNET.clone();
    let db = Arc::new(init_db(db_path).unwrap());

    let tree_externals = TreeExternals::new(
        db.clone(),
        Arc::new(BeaconConsensus::new(Arc::clone(&chain))),
        Factory::new(chain.clone()),
        Arc::clone(&chain),
    );

    let tree_config = BlockchainTreeConfig::default();
    let (canon_state_notification_sender, _receiver) =
        tokio::sync::broadcast::channel(tree_config.max_reorg_depth() as usize * 2);

    let blockchain_tree = ShareableBlockchainTree::new(
        BlockchainTree::new(tree_externals, canon_state_notification_sender, tree_config).unwrap(),
    );

    Ok(BlockchainProvider::new(
        ProviderFactory::new(Arc::clone(&db), Arc::clone(&chain)),
        blockchain_tree,
    )
    .unwrap())
}

/// EthApi
pub fn init_eth_api(client: RethClient, task_manager: TaskManager) -> RethApi {
    let tx_pool = init_pool(client.clone(), task_manager);

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    EthApi::new(
        client.clone(),
        tx_pool,
        NoopNetwork,
        state_cache.clone(),
        GasPriceOracle::new(client, GasPriceOracleConfig::default(), state_cache),
    )
}

// EthApi/Filter txPool
pub fn init_pool(client: RethClient, task_manager: TaskManager) -> RethTxPool {
    let chain = MAINNET.clone();

    reth_transaction_pool::Pool::eth_pool(
        EthTransactionValidator::new(client, chain, task_manager.executor(), 1),
        Default::default(),
    )
}

// Parity Traces
pub fn init_trace(
    client: RethClient,
    eth_api: RethApi,
    task_manager: TaskManager,
    max_tracing_requests: u32,
) -> RethTrace {
    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let task_spawn_handle: Box<dyn TaskSpawner> = Box::new(task_manager.executor());

    let tracing_call_guard = TracingCallGuard::new(max_tracing_requests);

    TraceApi::new(client, eth_api, state_cache, task_spawn_handle, tracing_call_guard)
}

// Geth Debug Traces
pub fn init_debug(
    client: RethClient,
    eth_api: RethApi,
    task_manager: TaskManager,
    max_tracing_requests: u32,
) -> RethDebug {
    let task_spawn_handle: Box<dyn TaskSpawner> = Box::new(task_manager.executor());

    let tracing_call_guard = TracingCallGuard::new(max_tracing_requests);

    DebugApi::new(client, eth_api, task_spawn_handle, tracing_call_guard)
}

// EthFilter
pub fn init_eth_filter(
    client: RethClient,
    max_logs_per_response: usize,
    task_manager: TaskManager,
) -> RethFilter {
    let task_spawn_handle: Box<dyn TaskSpawner> = Box::new(task_manager.executor());

    let tx_pool = init_pool(client.clone(), task_manager);

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    EthFilter::new(client, tx_pool, state_cache, max_logs_per_response, task_spawn_handle)
}
