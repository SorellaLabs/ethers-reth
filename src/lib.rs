// std
use eyre::Result;
use std::{fmt::Debug, path::Path, sync::Arc};

// ethers
use ethers::providers::{Middleware, MiddlewareError};

//Reth
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::ShareableBlockchainTree;
use reth_db::mdbx::{Env, WriteMap};
use reth_network_api::noop::NoopNetwork;
use reth_provider::providers::BlockchainProvider;
use reth_revm::Factory;
use reth_rpc::{eth::error::EthApiError, DebugApi, EthApi, EthFilter, TraceApi};
use reth_transaction_pool::{EthTransactionValidator, CoinbaseTipOrdering, Pool, PooledTransaction};
//Error
use jsonrpsee::types::ErrorObjectOwned;
use thiserror::Error;

pub mod init;
pub mod middleware;
pub mod type_conversions;
use tokio::runtime::Handle;

pub type RethClient = BlockchainProvider<
    Arc<Env<WriteMap>>,
    ShareableBlockchainTree<Arc<Env<WriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type RethTxPool = Pool<
    EthTransactionValidator<RethClient, PooledTransaction>,
    CoinbaseTipOrdering<PooledTransaction>,
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
    pub fn new<P: AsRef<Path>>(inner: M, db_path: P, handle: Handle) -> Result<Self> {
        let (reth_api, reth_filter, reth_trace, reth_debug) =
            Self::try_new(db_path.as_ref(), handle)?;
        Ok(Self { inner, reth_api, reth_filter, reth_trace, reth_debug })
    }

    pub fn reth_api(&self) -> &RethApi {
        &self.reth_api
    }
}
