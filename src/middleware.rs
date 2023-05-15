use crate::{utils::*, RethMiddleware, RethMiddlewareError};
use async_trait::async_trait;

// Ether rs Types
use ethers::{
    providers::{ProviderError, Middleware, MiddlewareError},
    types::{transaction::{eip2718::TypedTransaction, eip2930::AccessList}, Bytes},
};
use ethers::types::transaction::eip2930::AccessListWithGasUsed;

use ethers::types::BlockId as EthersBlockId;
use ethers::types::Transaction as EthersTransaction;
use ethers::types::TransactionReceipt as EthersTransactionReceipt;
use ethers::types::Address;
use ethers::types::NameOrAddress;
use ethers::types::Filter;
use ethers::types::U256 as EthersU256;
use ethers::types::H256 as EthersH256;
use ethers::types::Log;
use ethers::types::TxHash;
use ethers::types::FeeHistory;
use ethers::types::BlockNumber;
use ethers::types::EIP1186ProofResponse as EthersEIP1186ProofResponse;
// Reth Types
use reth_network_api::NetworkInfo;
use reth_provider::{BlockProvider, EvmEnvProvider, StateProviderFactory, BlockProviderIdExt, BlockIdProvider, HeaderProvider};
use reth_rpc::{eth::{EthApi, EthTransactions, *}, EthApiSpec};
use reth_rpc_api::EthApiServer;
use reth_transaction_pool::TransactionPool;
use reth_primitives::{BlockId, serde_helper::JsonStorageKey, H256};


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
        let res = self
            .reth_api
            .call(call_request, block_id, None)
            .await?
            .to_vec()
            .into();
        
        Ok(res)
    }

    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<EthersU256, Self::Error> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(|b| ethers_block_id_to_reth_block_id(b));
        let res = self
            .reth_api
            .estimate_gas(call_request, block_id)
            .await?;

        Ok(res.into())
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed, RethMiddlewareError<M>> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(|b| ethers_block_id_to_reth_block_id(b));
        let result = self
            .reth_api
            .create_access_list(call_request, block_id)
            .await
            .map_err(RethMiddlewareError::RethEthApiError)?;

        let converted_result = reth_access_list_with_gas_used_to_ethers(result);
        Ok(converted_result)
    }

    

    //TODO: implement filter type conversion into reth & log query & type reconversion 
    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
        Ok(self.reth_api.get_logs(filter).await?)
    }   


    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: EthersH256,
        block: Option<EthersBlockId>,
    ) -> Result<H256, Self::Error> {
        // convert `from` to `Address` and `block` to `Option<BlockId>`
        let from: Address = match from.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?.into(),
            NameOrAddress::Address(addr) => addr.into(),
        };
    
        // convert `location` to `JsonStorageKey`
        let index: JsonStorageKey = EthersU256::from_big_endian(location.as_bytes()).into();
    
        // convert `block` to `Option<BlockId>`
        let block: Option<BlockId> = block.map(ethers_block_id_to_reth_block_id);
    
        // call `storage_at`
        match self.reth_api.storage_at(from, index, block).await {
            Ok(value) => Ok(EthersH256::from_slice(value.as_bytes())),
            Err(e) => Err(RethMiddlewareError::RethEthApiError(e.into())),
        }
    }


    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<EthersTransaction>, ProviderError> {
        //let hash = ethers::types::H256::from_slice(transaction_hash.into().as_bytes());
        match self.reth_api.transaction_by_hash(transaction_hash).await {
            Ok(Some(tx)) => Ok(Some(reth_transaction_to_ethers(tx))),
            Ok(None) => Ok(None),
            Err(e) => Err(RethMiddlewareError::RethEthApiError(e.into())),
        }
    }

    async fn get_balance(
        &self,
        address: Address,
        blocknumber: u64,
    ) -> Result<U256, ProviderError> {
        let block_id = Some(BlockId::Number(blocknumber.into()));
        let balance = self.reth_api.balance(address, block_id).await?;
        Ok(balance.into())
    }

    async fn get_code(
        &self,
        address: Address,
        blocknumber: u64,
    ) -> Result<Bytes, ProviderError> {
        let block_id = Some(BlockId::Number(blocknumber.into()));
        let code = self.reth_api.get_code(address, block_id).await?;
        Ok(code.into())
    }

    async fn get_transaction_count(
        &self,
        address: Address,
        blocknumber: u64,
    ) -> Result<U256, ProviderError> {
        let block_id = Some(BlockId::Number(blocknumber.into()));
        let count = self.reth_api.transaction_count(address, block_id).await?;
        Ok(count.into())
    }


    async fn get_chainid(&self) -> Result<U256, ProviderError> {
        let chain_id = self.reth_api.chain_id().await?;
        Ok(chain_id.into())
    }

    async fn get_block_number(&self) -> Result<U256, ProviderError> {
        let block_number = self.reth_api.block_number().await?;
        Ok(block_number.into())
    }


    async fn fee_history<T: Into<U256> + Send + Sync>(
        &self,
        block_count: T,
        last_block: BlockNumber,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory, Self::Error> {
        let block_count = block_count.into();
        let last_block = last_block.into();
        let reward_percentiles = Some(reward_percentiles.to_vec());
        let fee_history = self
            .reth_api
            .fee_history(block_count, last_block, reward_percentiles)
            .await
            .map_err(RethMiddlewareError::RethEthApiError)?;
        Ok(fee_history)
    }

    /*async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<EthersTransactionReceipt>, Self::Error> {
        let block = block.into();
        let receipts = self
            .reth_api
            .block_receipts(block)
            .await
            .map_err(RethMiddlewareError::RethEthApiError)?;
        Ok(receipts)
    }
    */

    async fn get_proof<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        locations: Vec<EthersH256>,
        block: Option<EthersBlockId>,
    ) -> Result<EthersEIP1186ProofResponse, ProviderError> {
        let from: Address = match from.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await?.into(),
            NameOrAddress::Address(addr) => addr.into(),
        };
        let locations = locations
            .into_iter()
            .map(|location| JsonStorageKey::from(location.as_bytes()))
            .collect();

        let block_id = Some(BlockId::Number(block.into()));
        let proof = self
            .reth_api
            .get_proof(from, locations, block_id)
            .await
            .map_err(RethMiddlewareError::RethEthApiError)?;
        Ok(proof.into())
    }


    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<EthersTransactionReceipt>, ProviderError> {
        let hash = ethers::types::H256::from_slice(transaction_hash.into().as_bytes());
        match self.reth_api.transaction_receipt(hash).await {
            Ok(Some(receipt)) => Ok(Some(reth_transaction_receipt_to_ethers(receipt))),
            Ok(None) => Ok(None),
            Err(e) => Err(RethMiddlewareError::RethEthApiError(e.into())),
        }
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

