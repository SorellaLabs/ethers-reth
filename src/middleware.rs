use crate::{utils::*, RethMiddleware, RethMiddlewareError};
use async_trait::async_trait;
use ethers::{
    providers::Middleware,
    types::{transaction::eip2718::TypedTransaction, BlockId, Bytes},
};
use reth_network_api::NetworkInfo;
use reth_provider::{BlockProvider, EvmEnvProvider, StateProviderFactory};
use reth_rpc_api::EthApiServer;
use reth_transaction_pool::TransactionPool;

#[async_trait]
impl<M, Client, Pool, Network> Middleware for RethMiddleware<M, Client, Pool, Network>
where
    M: Middleware,
    Client: BlockProvider + EvmEnvProvider + StateProviderFactory + 'static,
    Pool: TransactionPool + Clone + 'static,
    Network: NetworkInfo + 'static,
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
            .await?
            .0
            .into())
    }
}
