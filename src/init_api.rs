
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
use std::sync::Arc;



pub type RethClient = BlockchainProvider<
    Arc<Env<NoWriteMap>>,
    ShareableBlockchainTree<Arc<Env<NoWriteMap>>, Arc<BeaconConsensus>, Factory>,
>;


pub type RethTxPool = Pool<
    EthTransactionValidator<
        BlockchainProvider<
            Arc<Env<NoWriteMap>>,
            ShareableBlockchainTree<Arc<Env<NoWriteMap>>, Arc<BeaconConsensus>, Factory>,
        >,
        PooledTransaction,
    >,
    CostOrdering<PooledTransaction>,
>;


pub type RethEthApi = EthApi<RethClient, RethTxPool, NoopNetwork>;




pub type testClient = BlockchainProvider<DB, Tree = ShareableBlockchainTree>;



pub fn new_reth_api<P: AsRef<std::path::Path>>(db_path: P) -> RethEthApi {
    let chain = Arc::new(MAINNET.clone());
    let db = Arc::new(Env::<NoWriteMap>::open(db_path.as_ref(), EnvKind::RO).unwrap());
    let consensus = Arc::new(BeaconConsensus::new(Arc::clone(&chain)));
    let tree_externals = TreeExternals::new(
        Arc::clone(&db),
        Arc::clone(&consensus),
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

    let shareable_db = ShareableDatabase::new(Arc::clone(&db), Arc::clone(&chain));
    let blockchain_db = BlockchainProvider::new(shareable_db, blockchain_tree).unwrap();


    let tx_pool = reth_transaction_pool::Pool::eth_pool(
        EthTransactionValidator::new(blockchain_db.clone(), chain),
        Default::default(),
    );
    let state_cache = EthStateCache::spawn(blockchain_db.clone(), EthStateCacheConfig::default());

    let gas_price_oracle = GasPriceOracle::new(blockchain_db.clone(), GasPriceOracleConfig::default(), state_cache.clone());

    let api = EthApi::new(blockchain_db, tx_pool, NoopNetwork, state_cache, gas_price_oracle);
    api
}


<<<<<<< HEAD
//pub fn new(blockchain_db: B, tx_pool: P, eth_cache: EthStateCache, gas_oracle: GasPriceOracle<B>) -> RethEthApi {

//}

=======
>>>>>>> 2a32ea003257a9aa802cd81c2bb84345e340a673
