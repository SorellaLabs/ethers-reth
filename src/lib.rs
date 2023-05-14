pub mod init_api;
pub mod middleware;
mod utils;
use ethers::providers::{Middleware, MiddlewareError};
use init_api::RethEthApi;
use jsonrpsee::types::*;
use thiserror::Error;



#[derive(Clone)]
pub struct RethMiddleware<M> {
    inner: M,
    reth_api: RethEthApi,
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
    pub fn new(inner: M, reth_api: RethEthApi) -> Self {
        Self { inner, reth_api }
    }

    pub fn reth_api(&self) -> &RethEthApi {
        &self.reth_api
    }
}
