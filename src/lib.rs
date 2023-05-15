pub mod reth_api;
pub mod middleware;
mod utils;
use std::{sync::Arc, path::Path};
pub mod defaults;
pub mod reth_filter;


use defaults::default_client;
use ethers::providers::{Middleware, MiddlewareError};
use reth_api::{NodeEthApi, default_eth_api};
use jsonrpsee::types::*;
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::ShareableBlockchainTree;
use reth_db::mdbx::{NoWriteMap, Env};
use reth_filter::{NodeEthFilter, default_eth_filter};
use reth_provider::providers::BlockchainProvider;
use reth_revm::Factory;
use reth_transaction_pool::{PooledTransaction, EthTransactionValidator, Pool, CostOrdering};
use thiserror::Error;


#[derive(Clone)]
pub struct RethMiddleware<M> {
    inner: M,
    reth_api: NodeEthApi,
    reth_filter: NodeEthFilter
}

impl<M: std::fmt::Debug> std::fmt::Debug
    for RethMiddleware<M>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RethMiddleware")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

#[derive(Error, Debug)]
pub enum RethMiddlewareError<M: Middleware> {
    /// An error occured in one of the middlewares.
    #[error("{0}")]
    MiddlewareError(M::Error),
    #[error(transparent)]
    RethEthApiError(#[from] ErrorObjectOwned),
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
    pub fn new(inner: M, reth_api: NodeEthApi, reth_filter: NodeEthFilter) -> Self {
        Self { inner, reth_api, reth_filter }
    }

    pub fn default(inner: M, db_path: &Path) -> Self {

        let client = default_client(db_path);

        Self { 
            inner, 
            reth_api: default_eth_api(client.clone()), 
            reth_filter: default_eth_filter(client.clone(), 1000) 
        }
    }

    pub fn reth_api(&self) -> &NodeEthApi {
        &self.reth_api
    }

    pub fn reth_filter(&self) -> &NodeEthFilter {
        &self.reth_filter
    }
}
