pub mod middleware;
mod utils;
use std::sync::Arc;
use jsonrpsee::types::ErrorObjectOwned;
use reth_rpc::EthFilter;
use thiserror::Error;



use ethers::providers::{Middleware, MiddlewareError};
use ethers::prelude::Client;

use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::ShareableBlockchainTree;
use reth_db::mdbx::{Env, NoWriteMap};
use reth_provider::providers::BlockchainProvider;
use reth_revm::Factory;
use reth_rpc::eth::error::EthApiError;
use reth_transaction_pool::{CostOrdering, EthTransactionValidator, Pool, PooledTransaction};
use async_trait::async_trait;
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
    EthApi,
};
use std::path::Path;


pub type NodeClient = BlockchainProvider<
    Arc<Env<NoWriteMap>>,
    ShareableBlockchainTree<Arc<Env<NoWriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type NodeTxPool =
    Pool<EthTransactionValidator<NodeClient, PooledTransaction>, CostOrdering<PooledTransaction>>;

pub type NodeEthApi = EthApi<NodeClient, NodeTxPool, NoopNetwork>;
pub type NodeEthFilter = EthFilter<NodeClient, NodeTxPool>;


#[derive(Clone)]
pub struct RethMiddleware<M> {
    inner: M,
    reth_api: NodeEthApi,
    reth_filter: NodeEthFilter,
}

impl<M: std::fmt::Debug> std::fmt::Debug for RethMiddleware<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RethMiddleware").field("inner", &self.inner).finish_non_exhaustive()
    }
}

#[derive(Error, Debug)]
pub enum RethMiddlewareError<M: Middleware> {
    /// An error occured in one of the middlewares.
    #[error("{0}")]
    MiddlewareError(M::Error),

    /// An error occurred in the Reth API.
    #[error(transparent)]
    RethApiError(#[from] ErrorObjectOwned),
}

impl<M: Middleware> MiddlewareError for RethMiddlewareError<M> {
    type Inner = M::Error;

    fn from_err(e: Self::Inner) -> Self {
        RethMiddlewareError::MiddlewareError(e)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            RethMiddlewareError::MiddlewareError(e) => Some(e),
            _ => None,
        }
    }
}

impl<M> RethMiddleware<M>
where
    M: Middleware,
{
    pub fn new(inner: M, db_path: &Path) -> Self {

        let client = Self::init_client(db_path);
        // EthApi -> EthApi<Client, Pool, Network>
        let api = Self::init_eth_api(client.clone());
        // EthApi -> EthFilter<Client, Pool>
        let filter = Self::init_eth_filter(client.clone(), 1000);

        Self { inner, reth_api: api, reth_filter: filter}
    }

    pub fn reth_api(&self) -> &NodeEthApi {
        &self.reth_api
    }

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
        let tx_pool = Self::init_pool(client.clone());

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

        let tx_pool = Self::init_pool(client.clone());

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
}
