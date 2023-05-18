// std
use std::{path::Path, sync::Arc};

// ethers
use ethers::providers::{Middleware, MiddlewareError};

//Reth
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::ShareableBlockchainTree;
use reth_db::mdbx::{Env, NoWriteMap};
use reth_network_api::test_utils::NoopNetwork;
use reth_provider::providers::BlockchainProvider;
use reth_revm::Factory;
use reth_rpc::{eth::error::EthApiError, EthApi, EthFilter, TraceApi};
use reth_transaction_pool::{CostOrdering, EthTransactionValidator, Pool, PooledTransaction};

//Error
use jsonrpsee::types::ErrorObjectOwned;
use thiserror::Error;
// own modules
pub mod init;
pub mod middleware;
pub mod utils;
use init::{init_client, init_eth_api, init_eth_filter, init_trace};

pub type RethClient = BlockchainProvider<
    Arc<Env<NoWriteMap>>,
    ShareableBlockchainTree<Arc<Env<NoWriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type RethTxPool =
    Pool<EthTransactionValidator<RethClient, PooledTransaction>, CostOrdering<PooledTransaction>>;

pub type RethApi = EthApi<RethClient, RethTxPool, NoopNetwork>;
pub type RethFilter = EthFilter<RethClient, RethTxPool>;
pub type RethTrace = TraceApi<RethClient, RethApi>;

#[derive(Clone)]
pub struct RethMiddleware<M> {
    inner: M,
    reth_api: RethApi,
    reth_filter: RethFilter,
    reth_trace: RethTrace,
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

    /// An error occurred in the Eth API.
    #[error(transparent)]
    EthApiError(#[from] EthApiError),

    /// A trace was expected but none was found.
    #[error("Missing trace")]
    MissingTrace,
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
        let client = init_client(db_path);
        // Create a runtime here and use Arc to share it across functions
        let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());

        // EthApi -> EthApi<Client, Pool, Network>
        let api = init_eth_api(client.clone());
        // EthFilter -> EthFilter<Client, Pool>
        let filter = init_eth_filter(client.clone(), 1000, rt.clone());
        let trace = init_trace(client, api.clone(), rt, 10);

        Self { inner, reth_api: api, reth_filter: filter, reth_trace: trace }
    }

    pub fn reth_api(&self) -> &RethApi {
        &self.reth_api
    }
}
