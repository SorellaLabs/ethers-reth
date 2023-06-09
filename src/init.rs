use crate::{provider::view, RethApi, RethClient, RethFilter, RethTrace, RethTxPool};
use eyre::Context;
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};
use reth_db::{
    mdbx::{Env, WriteMap},
    tables, DatabaseError,
};
use reth_network_api::test_utils::NoopNetwork;
use reth_primitives::MAINNET;
use reth_provider::{providers::BlockchainProvider, ShareableDatabase};
use reth_revm::Factory;
use reth_rpc::{
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    EthApi, EthFilter, TraceApi, TracingCallGuard,
};
use reth_tasks::{TaskManager, TaskSpawner};
use reth_transaction_pool::EthTransactionValidator;
use std::{fmt::Debug, path::Path, sync::Arc};

/// Opens up an existing database at the specified path.
pub fn init_db<P: AsRef<Path> + Debug>(path: P) -> eyre::Result<Env<WriteMap>> {
    std::fs::create_dir_all(path.as_ref())?;
    println!("{:?}", path);
    let db = reth_db::mdbx::Env::<reth_db::mdbx::WriteMap>::open(
        path.as_ref(),
        reth_db::mdbx::EnvKind::RO,
    )?;

    view(&db, |tx| {
        for table in tables::TABLES.iter().map(|(_, name)| name) {
            tx.inner.open_db(Some(table)).wrap_err("Could not open db.").unwrap();
        }
    })?;

    Ok(db)
}

// EthApi/Filter Client
pub fn init_client(db_path: &Path) -> Result<RethClient, DatabaseError> {
    let chain = Arc::new(MAINNET.clone());
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
        ShareableDatabase::new(Arc::clone(&db), Arc::clone(&chain)),
        blockchain_tree,
    )
    .unwrap())
}

/// EthApi
pub fn init_eth_api(client: RethClient) -> RethApi {
    let tx_pool = init_pool(client.clone());

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
pub fn init_pool(client: RethClient) -> RethTxPool {
    let chain = Arc::new(MAINNET.clone());

    reth_transaction_pool::Pool::eth_pool(
        EthTransactionValidator::new(client, chain),
        Default::default(),
    )
}

// RethTrace
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

// EthFilter
pub fn init_eth_filter(
    client: RethClient,
    max_logs_per_response: usize,
    task_manager: TaskManager,
) -> RethFilter {
    let tx_pool = init_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let task_spawn_handle: Box<dyn TaskSpawner> = Box::new(task_manager.executor());

    EthFilter::new(client, tx_pool, state_cache, max_logs_per_response, task_spawn_handle)
}
