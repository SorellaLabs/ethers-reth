use crate::{utils::*, RethMiddleware, RethMiddlewareError};
use async_trait::async_trait;


// Ether rs Types
use ethers::{
    providers::{ProviderError, Middleware},
    types::{transaction::eip2718::TypedTransaction, BlockId, Bytes},
};
use ethers::types::transaction::eip2930::AccessListWithGasUsed;
use ethers::types::*;

// Reth Types
use reth_network_api::NetworkInfo;
use reth_provider::{BlockProvider, EvmEnvProvider, StateProviderFactory, BlockProviderIdExt, BlockIdProvider, HeaderProvider};
use reth_rpc::{eth::{EthApi, EthTransactions, *}, EthApiSpec};
use reth_rpc_api::EthApiServer;
use reth_transaction_pool::TransactionPool;


// Std Lib
use std::fmt::Debug;
use serde::{de::DeserializeOwned, Serialize};


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

    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(|b| ethers_block_id_to_reth_block_id(b));
        Ok(self
            .reth_api
            .estimate_gas(call_request, block_id)
            .await?
            .into())
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed, ProviderError> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(|b| ethers_block_id_to_reth_block_id(b));
        Ok(self
            .reth_api
            .create_access_list(call_request, block_id)
            .await?
            .0)
    }


    //TODO: implement filter type conversion into reth & log query & type reconversion 
    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
        Ok(self.reth_api.get_logs(filter).await?)
    }   


    /// Get the storage of an address for a particular slot location
    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: H256,
        block: Option<BlockId>,
    ) -> Result<H256, Self::Error> {
        self.reth_api.storage(from, location, block).await
    }


}



    

impl<M> RethMiddleware<M>
    where
        M: Middleware,
{
    pub async fn request<T, R>(&self, method: &str, params: T) -> Result<R, ProviderError>
    where
        T: Debug + Serialize + Send + Sync,
        R: Serialize + DeserializeOwned + Debug + Send,
    
    {
        match method {
            "eth_call" => {
                let tx = serde_json::from_value::<&TypedTransaction>(params)?;
                let block = serde_json::from_value::<Option<BlockId>>(params)?;
                let res = self.call(tx, block).await?;
                Ok(res)
            }
            "eth_estimateGas" => {
                let tx = serde_json::from_value::<&TypedTransaction>(params)?;
                let block = serde_json::from_value::<Option<BlockId>>(params)?;
                let res = self.estimate_gas(tx, block).await?;
                Ok(res)
            }
            "eth_createAcessList" => {
                let tx = serde_json::from_value::<&TypedTransaction>(params)?;
                let block = serde_json::from_value::<Option<BlockId>>(params)?;
                let res = self.create_access_list(tx, block).await?;
                Ok(res)
            }
            _ => ProviderError::from(ProviderError::UnsupportedRPC(method.to_string())),
        }
    }
}

