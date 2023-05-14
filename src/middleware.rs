use crate::{utils::*, RethMiddleware, RethMiddlewareError};
use async_trait::async_trait;
use ethers::{
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, BlockId, Bytes},
};
use reth_network_api::NetworkInfo;
use reth_provider::{BlockProvider, EvmEnvProvider, StateProviderFactory, BlockProviderIdExt, BlockIdProvider, HeaderProvider};
use reth_rpc::{eth::{EthApi, EthTransactions, *}, EthApiSpec};
use reth_rpc_api::EthApiServer;
use reth_transaction_pool::TransactionPool;

#[async_trait]
impl<M> Middleware for RethMiddleware<M>
where
    M: Middleware,
{
    type Error = RethMiddlewareError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(|b| ethers_block_id_to_reth_block_id(b));
        Ok(self
            .reth_api
            .call(call_request, block_id, None)
            .await.unwrap()
            .0
            .into())
    }
}

