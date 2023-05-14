
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
    eth::cache::{EthStateCache, EthStateCacheConfig},
    EthApi,
};
use reth_transaction_pool::{CostOrdering, EthTransactionValidator, Pool, PooledTransaction};
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
pub type RethNetwork = NoopNetwork;
pub type RethEthApi = EthApi<RethClient, RethTxPool, RethNetwork>;

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
    let blockchain_db = BlockchainProvider::new(shareable_db, blockchain_tree);

    let transaction_pool = reth_transaction_pool::Pool::eth_pool(
        EthTransactionValidator::new(blockchain_db.clone(), chain),
        Default::default(),
    );
    let state_cache = EthStateCache::spawn(blockchain_db.clone(), EthStateCacheConfig::default());

    let api = EthApi::new(blockchain_db, transaction_pool, NoopNetwork, state_cache);
    api
}


Provider::<IPC>::new("/tmp/geth.ipc", None, None).unwrap();