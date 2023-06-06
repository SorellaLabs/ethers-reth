use crate::{RethApi, RethClient, RethFilter, RethTrace, RethTxPool};
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};
use reth_db::{mdbx::{Env, EnvKind, WriteMap, DatabaseFlags, EnvironmentKind}, DatabaseError, TableType, TABLES};
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
use std::{path::Path, sync::Arc};


// EthApi/Filter Client
pub fn init_client(db_path: &Path) -> RethClient {
    let chain = Arc::new(MAINNET.clone());
    let open_db = Env::<WriteMap>::open(db_path.as_ref(), EnvKind::RO).unwrap();
    open_db.create_tables().unwrap();
    let db = Arc::new(open_db);


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
        BlockchainTree::new(tree_externals, canon_state_notification_sender.clone(), tree_config)
            .unwrap(),
    );

    BlockchainProvider::new(
        ShareableDatabase::new(Arc::clone(&db), Arc::clone(&chain)),
        blockchain_tree,
    )
    .unwrap()
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
    rt: Arc<tokio::runtime::Runtime>,
    max_tracing_requests: u32,
) -> RethTrace {
    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let task_manager = TaskManager::new(rt.handle().clone());
    let task_spawn_handle: Box<dyn TaskSpawner> = Box::new(task_manager.executor());

    let tracing_call_guard = TracingCallGuard::new(max_tracing_requests);

    TraceApi::new(client, eth_api, state_cache, task_spawn_handle, tracing_call_guard)
}

// EthFilter
pub fn init_eth_filter(
    client: RethClient,
    max_logs_per_response: usize,
    rt: Arc<tokio::runtime::Runtime>,
) -> RethFilter {
    let tx_pool = init_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let task_manager = TaskManager::new(rt.handle().clone());
    let task_spawn_handle: Box<dyn TaskSpawner> = Box::new(task_manager.executor());

    EthFilter::new(client, tx_pool, state_cache, max_logs_per_response, task_spawn_handle)
}
