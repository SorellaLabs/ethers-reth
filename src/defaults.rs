use async_trait::async_trait;
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};
use reth_db::mdbx::{Env, EnvKind, NoWriteMap};
use reth_network_api::test_utils::NoopNetwork;
use reth_primitives::MAINNET;
use reth_provider::{providers::BlockchainProvider, ShareableDatabase};
use reth_revm::Factory;
use reth_rpc::{
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    EthApi,
};
use reth_transaction_pool::{CostOrdering, EthTransactionValidator, Pool, PooledTransaction};
use std::{path::Path, sync::Arc};

pub type NodeClient = BlockchainProvider<
    Arc<Env<NoWriteMap>>,
    ShareableBlockchainTree<Arc<Env<NoWriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type NodeTxPool =
    Pool<EthTransactionValidator<NodeClient, PooledTransaction>, CostOrdering<PooledTransaction>>;

// default client
pub fn default_client(db_path: &Path) -> NodeClient {
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

// default txPool
pub fn default_pool(client: NodeClient) -> NodeTxPool {
    let chain = Arc::new(MAINNET.clone());

    let tx_pool = reth_transaction_pool::Pool::eth_pool(
        EthTransactionValidator::new(client, chain),
        Default::default(),
    );

    tx_pool
}
