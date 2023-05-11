pub mod init_api;
pub mod middleware;
mod utils;

use ethers::providers::{Middleware, MiddlewareError};
use jsonrpsee::core::error::Error as RpcError;
use reth_network_api::NetworkInfo;
use reth_provider::{BlockProvider, EvmEnvProvider, StateProviderFactory};
use reth_rpc::EthApi;
use reth_transaction_pool::TransactionPool;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RethMiddlewareError<M: Middleware> {
    /// An error occured in one of the middlewares.
    #[error("{0}")]
    MiddlewareError(M::Error),
    #[error(transparent)]
    RethEthApiError(#[from] RpcError),
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

pub struct RethMiddleware<M, Client, Pool, Network> {
    inner: M,
    reth_api: EthApi<Client, Pool, Network>,
}

impl<M: std::fmt::Debug, Client, Pool, Network> std::fmt::Debug
    for RethMiddleware<M, Client, Pool, Network>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RethMiddleware")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl<M, Client, Pool, Network> RethMiddleware<M, Client, Pool, Network>
where
    M: Middleware,
    Client: BlockProvider + EvmEnvProvider + StateProviderFactory + 'static,
    Pool: TransactionPool + Clone + 'static,
    Network: NetworkInfo + 'static,
{
    pub fn new(inner: M, reth_api: EthApi<Client, Pool, Network>) -> Self {
        Self { inner, reth_api }
    }

    pub fn reth_api(&self) -> &EthApi<Client, Pool, Network> {
        &self.reth_api
    }
}
