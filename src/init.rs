use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::ShareableBlockchainTree;
use reth_db::mdbx::{Env, NoWriteMap};
use reth_provider::providers::BlockchainProvider;
use reth_revm::Factory;
use reth_transaction_pool::EthTransactionValidator;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig,
};
use reth_db::mdbx::EnvKind;
use reth_network_api::test_utils::NoopNetwork;
use reth_primitives::MAINNET;
use reth_provider::ShareableDatabase;
use reth_rpc::{
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    EthApi, EthFilter,
};
use std::{path::Path, sync::Arc};

use crate::{NodeClient, NodeEthApi, NodeEthFilter, NodeTxPool};
   
// EthApi/Filter Client
pub fn init_client(db_path: &Path) -> NodeClient {
    let chain = Arc::new(MAINNET.clone());
    let db = Arc::new(Env::<NoWriteMap>::open(db_path.as_ref(), EnvKind::RO).unwrap());

    let tree_externals = TreeExternals::new(
        Arc::clone(&db),
        Arc::new(BeaconConsensus::new(Arc::clone(&chain))),
        Factory::new(Arc::clone(&chain)),
        Arc::clone(&chain),
    );

    let tree_config = BlockchainTreeConfig::default();
    let (canon_state_notification_sender, _receiver) =
        tokio::sync::broadcast::channel(tree_config.max_reorg_depth() as usize * 2);

    let blockchain_tree = ShareableBlockchainTree::new(
        BlockchainTree::new(tree_externals, canon_state_notification_sender.clone(), tree_config)
            .unwrap(),
    );

    let blockchain_db = BlockchainProvider::new(
        ShareableDatabase::new(Arc::clone(&db), Arc::clone(&chain)),
        blockchain_tree,
    )
    .unwrap();

    blockchain_db
}


/// EthApi
pub fn init_eth_api(client: NodeClient) -> NodeEthApi {
    let tx_pool = init_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let api = EthApi::new(
        client.clone(),
        tx_pool,
        NoopNetwork,
        state_cache,
        GasPriceOracle::new(client.clone(), GasPriceOracleConfig::default(), state_cache.clone()),
    );

    api
}

/// EthFilter
pub fn init_eth_filter(client: NodeClient, max_logs_per_response: usize) -> NodeEthFilter {

    let tx_pool = init_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let filter = EthFilter::new(
        client, 
        tx_pool, 
        state_cache, 
        max_logs_per_response
    );
    
    filter
}

// EthApi/Filter txPool
pub fn init_pool(client: NodeClient) -> NodeTxPool {
    let chain = Arc::new(MAINNET.clone());

    let tx_pool = reth_transaction_pool::Pool::eth_pool(
        EthTransactionValidator::new(client, chain),
        Default::default(),
    );

    tx_pool
}