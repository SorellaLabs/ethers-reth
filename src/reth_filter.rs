
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
    eth::{cache::{EthStateCache, EthStateCacheConfig}, gas_oracle::{GasPriceOracle, GasPriceOracleConfig}},
    EthApi,
};
use reth_transaction_pool::{EthTransactionValidator, Pool, PooledTransaction, CostOrdering};
use std::{sync::Arc, path::Path};


pub type NodeEthFilter = EthFilter<NodeClient, NodeTxPool>;


// trait for reth EthApi
#[async_trait]
pub trait RethEthFilter {

    fn default(db_path: &Path) -> Self;

}


impl RethEthApi for EthFilter<NodeClient, NodeTxPool> {

    /// default implementation
    fn default(db_path: &Path) -> Self {
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
            BlockchainTree::new(
                tree_externals,
                canon_state_notification_sender.clone(),
                tree_config,
            )
            .unwrap(),
        );

        let blockchain_db = BlockchainProvider::new(
            ShareableDatabase::new(Arc::clone(&db), Arc::clone(&chain)),
            blockchain_tree
        ).unwrap();

        let tx_pool = reth_transaction_pool::Pool::eth_pool(
            EthTransactionValidator::new(blockchain_db.clone(), chain),
            Default::default(),
        );

        let state_cache = EthStateCache::spawn(blockchain_db.clone(), EthStateCacheConfig::default());

        let api = EthApi::new(
            blockchain_db, 
            tx_pool, 
            NoopNetwork, 
            state_cache, 
            GasPriceOracle::new(blockchain_db.clone(), GasPriceOracleConfig::default(), state_cache.clone())
        );
        
        api
    }

}
