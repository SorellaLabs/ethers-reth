// std
use eyre::Result;
use std::{fmt::Debug, path::Path, sync::Arc};

// ethers
use ethers::providers::{Middleware, MiddlewareError};

//Reth
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::ShareableBlockchainTree;
use reth_db::mdbx::{Env, WriteMap};
use reth_network_api::test_utils::NoopNetwork;
use reth_provider::providers::BlockchainProvider;
use reth_revm::Factory;
use reth_rpc::{eth::error::EthApiError, DebugApi, EthApi, EthFilter, TraceApi};
use reth_tasks::TaskManager;
use reth_transaction_pool::{EthTransactionValidator, GasCostOrdering, Pool, PooledTransaction};
//Error
use jsonrpsee::types::ErrorObjectOwned;
use thiserror::Error;
// own modules
pub mod init;
pub mod middleware;
pub mod type_conversions;
use init::{init_client, init_debug, init_eth_api, init_eth_filter, init_trace};
use tokio::runtime::Handle;

pub type RethClient = BlockchainProvider<
    Arc<Env<WriteMap>>,
    ShareableBlockchainTree<Arc<Env<WriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type RethTxPool = Pool<
    EthTransactionValidator<RethClient, PooledTransaction>,
    GasCostOrdering<PooledTransaction>,
>;

pub type RethApi = EthApi<RethClient, RethTxPool, NoopNetwork>;
pub type RethFilter = EthFilter<RethClient, RethTxPool>;
pub type RethTrace = TraceApi<RethClient, RethApi>;
pub type RethDebug = DebugApi<RethClient, RethApi>;

#[derive(Clone)]
pub struct RethMiddleware<M> {
    inner: M,
    reth_api: RethApi,
    reth_filter: RethFilter,
    reth_trace: RethTrace,
    reth_debug: RethDebug,
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

    #[error("Chain Id unavailable")]
    ChainIdUnavailable,
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
    pub fn new<P: AsRef<Path>>(inner: M, db_path:P, handle: Handle) -> Result<Self> {
        let client = init_client(db_path)?;

        // EthApi -> EthApi<Client, Pool, Network>
        let api = init_eth_api(client.clone(), TaskManager::new(handle.clone()));
        // EthFilter -> EthFilter<Client, Pool>
        let filter = init_eth_filter(client.clone(), 1000, TaskManager::new(handle.clone()));
        let trace = init_trace(client.clone(), api.clone(), TaskManager::new(handle.clone()), 10);
        let debug = init_debug(client, api.clone(), TaskManager::new(handle), 10);

        Ok(Self { inner, reth_api: api, reth_filter: filter, reth_trace: trace, reth_debug: debug })
    }

    pub fn reth_api(&self) -> &RethApi {
        &self.reth_api
    }
}
