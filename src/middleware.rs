use crate::{
    utils::{convert_block_number_to_block_number_or_tag, ethers_block_id_to_reth_block_id, *},
    RethMiddleware, RethMiddlewareError,
};
use async_trait::async_trait;

// Ether rs Types
use ethers::{
    providers::Middleware,
    types::{
        transaction::{
            eip2718::TypedTransaction,
            eip2930::AccessListWithGasUsed as EthersAccessListWithGasUsed,
        },
        Address as EthersAddress, Block as EthersBlock, BlockId as EthersBlockId,
        BlockNumber as EthersBlocKNumber, BlockNumber as EthersBlockNumber, Bytes as EthersBytes,
        EIP1186ProofResponse as EthersEIP1186ProofResponse, FeeHistory as EthersFeeHistory,
        Filter as EthersFilter, Log as EthersLog, NameOrAddress, Trace as EthersTrace,
        Transaction as EthersTransaction, TransactionReceipt as EthersTransactionReceipt,
        TxHash as EthersTxHash, H256 as EthersH256, U256 as EthersU256, U64 as EthersU64,
    },
};

// Reth Types
use reth_rpc::EthApiSpec;
use reth_rpc_api::{EthApiServer, EthFilterApiServer};
use reth_rpc_types::Filter;

use reth_primitives::{serde_helper::JsonStorageKey, BlockId};

impl<M> RethMiddleware<M>
where
    Self: EthApiServer + EthApiSpec + 'static,
    M: Middleware,
{
    async fn get_address<T: Into<NameOrAddress>>(
        &self,
        who: T,
    ) -> Result<EthersAddress, <Self as Middleware>::Error> {
        match who.into() {
            NameOrAddress::Name(ens_name) => self.resolve_name(&ens_name).await,
            NameOrAddress::Address(addr) => Ok(addr.into()),
        }
    }
}

#[async_trait]
impl<M> Middleware for RethMiddleware<M>
where
    Self: EthApiServer + EthApiSpec + 'static,
    M: Middleware,
{
    type Error = RethMiddlewareError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    // Call related methods
    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<EthersBlockId>,
    ) -> Result<EthersBytes, Self::Error> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(ethers_block_id_to_reth_block_id);

        Ok(self.reth_api.call(call_request, block_id, None).await?.to_vec().into())
    }

    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<EthersBlockId>,
    ) -> Result<EthersU256, Self::Error> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(ethers_block_id_to_reth_block_id);

        Ok(self.reth_api.estimate_gas(call_request, block_id).await?.into())
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<EthersBlockId>,
    ) -> Result<EthersAccessListWithGasUsed, Self::Error> {
        let call_request = ethers_typed_transaction_to_reth_call_request(tx);
        let block_id = block.map(ethers_block_id_to_reth_block_id);

        let result = self.reth_api.create_access_list(call_request, block_id).await?;

        Ok(reth_access_list_with_gas_used_to_ethers(result))
    }

    // State related methods

    async fn get_storage_at<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        location: EthersH256,
        block: Option<EthersBlockId>,
    ) -> Result<EthersH256, Self::Error> {
        // convert `from` to `Address`
        let from = self.get_address(from).await?;
        // convert `location` to `JsonStorageKey`
        let index: JsonStorageKey = convert_location_to_json_key(location);
        // convert `block` to `Option<BlockId>`
        let block_id = block.map(ethers_block_id_to_reth_block_id);

        // call `storage_at` and convert the result
        Ok(self.reth_api.storage_at(from.into(), index, block_id).await?.into())
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<EthersBlockId>,
    ) -> Result<EthersBytes, Self::Error> {
        let at = self.get_address(at).await?;

        let block_id = block.map(|b| ethers_block_id_to_reth_block_id(b));
        let code = self.reth_api.get_code(at.into(), block_id).await?;
        // Convert to EthersBytes
        Ok(code.to_vec().into())
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<EthersBlockId>,
    ) -> Result<EthersU256, Self::Error> {
        let from = self.get_address(from).await?;

        let block_id = block.map(|b| ethers_block_id_to_reth_block_id(b));
        Ok(self.reth_api.balance(from.into(), block_id).await?.into())
    }

    async fn get_proof<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        locations: Vec<EthersH256>,
        block: Option<EthersBlockId>,
    ) -> Result<EthersEIP1186ProofResponse, RethMiddlewareError<M>> {
        let from = self.get_address(from).await?;

        let locations = locations
            .into_iter()
            .map(|location| JsonStorageKey::from(convert_location_to_json_key(location)))
            .collect();

        let block_id = Some(ethers_block_id_to_reth_block_id(block.unwrap()));
        let proof = self.reth_api.get_proof(from.into(), locations, block_id).await?;

        Ok(reth_proof_to_ethers(proof))
    }

    async fn fee_history<T: Into<EthersU256> + Send + Sync>(
        &self,
        block_count: T,
        last_block: EthersBlocKNumber,
        reward_percentiles: &[f64],
    ) -> Result<EthersFeeHistory, Self::Error> {
        let block_count = block_count.into();
        let last_block =
            ethers_block_id_to_reth_block_id(reth_primitives::rpc::BlockId::Number(last_block));
        let reward_percentiles = Some(reward_percentiles.to_vec());

        let reth_fee_history = self
            .reth_api
            .fee_history(ToReth::into_reth(block_count), last_block, reward_percentiles)
            .await?;

        Ok(reth_fee_history_to_ethers(reth_fee_history))
    }

    // Chain Info

    async fn get_chainid(&self) -> Result<EthersU256, RethMiddlewareError<M>> {
        let chain_id = EthApiServer::chain_id(&self.reth_api).await?;
        Ok(chain_id.unwrap().into_ethers())
    }

    async fn get_block_number(&self) -> Result<EthersU64, RethMiddlewareError<M>> {
        let block_number = self.reth_api.block_number()?;
        Ok(block_number.into_ethers())
    }

    /*async fn get_block_receipts<T: Into<EthersBlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<EthersTransactionReceipt>, Self::Error> {
        let block = block.into();
        let receipts = self
            .reth_api
            .block_receipts(block)
            .await?;

        Ok(receipts)
    }
    */

    // Transaction

    async fn get_transaction<T: Send + Sync + Into<EthersTxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<EthersTransaction>, Self::Error> {
        let maybe_transaction =
            self.reth_api.transaction_by_hash(transaction_hash.into().into()).await?;

        match maybe_transaction {
            Some(reth_tx) => Ok(Some(reth_rpc_transaction_to_ethers(reth_tx))),
            None => Ok(None),
        }
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<EthersTxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<EthersTransactionReceipt>, RethMiddlewareError<M>> {
        let hash = ethers::types::H256::from_slice(transaction_hash.into().as_bytes());
        match self.reth_api.transaction_receipt(hash.into()).await? {
            Some(receipt) => Ok(Some(reth_transaction_receipt_to_ethers(receipt))),
            None => Ok(None),
        }
    }

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<EthersBlockId>,
    ) -> Result<EthersU256, Self::Error> {
        let from = self.get_address(from).await?;

        let block_id = block.map(ethers_block_id_to_reth_block_id);
        Ok(self.reth_api.transaction_count(from.into(), block_id).await?.into())
    }

    // Blocks

    async fn get_block<T: Into<EthersBlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<EthersBlock<EthersTxHash>>, Self::Error> {
        let block_id = block_hash_or_number.into();

        let block = match block_id {
            EthersBlockId::Hash(hash) => self.reth_api.block_by_hash(hash.into(), false).await?,
            EthersBlockId::Number(num) => self.reth_api.block_by_number(num.into(), false).await?,
        };

        Ok(block.map(|block| hash_rich_block_to_ethers(block)))
    }

    async fn get_uncle<T: Into<EthersBlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: EthersU64,
    ) -> Result<Option<EthersBlock<EthersTxHash>>, Self::Error> {
        let block_id = block_hash_or_number.into();

        let block = match block_id {
            EthersBlockId::Hash(hash) => {
                self.reth_api
                    .uncle_by_block_hash_and_index(hash.into(), idx.as_usize().into())
                    .await?
            }
            EthersBlockId::Number(num) => {
                self.reth_api
                    .uncle_by_block_number_and_index(num.into(), idx.as_usize().into())
                    .await?
            }
        };

        Ok(block.map(|block| hash_rich_block_to_ethers(block)))
    }

    async fn get_block_with_txs<T: Into<EthersBlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<EthersBlock<EthersTransaction>>, Self::Error> {
        let block_id = block_hash_or_number.into();

        let block = match block_id {
            EthersBlockId::Hash(hash) => self.reth_api.block_by_hash(hash.into(), true).await?,
            EthersBlockId::Number(num) => self.reth_api.block_by_number(num.into(), true).await?,
        };

        Ok(block.map(|block| full_rich_block_to_ethers(block)))
    }

    // Logs

    async fn get_logs(&self, filter: &EthersFilter) -> Result<Vec<EthersLog>, Self::Error> {
        let to_reth_filter: Filter = ethers_filter_to_reth_filter(filter);
        let reth_logs = self.reth_filter.logs(to_reth_filter).await?;
        let ethers_logs = reth_logs.into_iter().map(|log| log.into_ethers()).collect();
        Ok(ethers_logs)
    }

    // Tracing

    async fn trace_transaction(
        &self,
        tx_hash: EthersTxHash,
    ) -> Result<Vec<EthersTrace>, Self::Error> {
        let trace = self.reth_trace.trace_transaction(tx_hash.into()).await?;

        // Convert each LocalizedTransactionTrace to an EthersTrace
        let ethers_traces: Vec<EthersTrace> = trace
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(|trace| reth_trace_to_ethers(trace))
            .collect();

        Ok(ethers_traces)
    }

    async fn trace_block(&self, block: EthersBlockNumber) -> Result<Vec<EthersTrace>, Self::Error> {
        let block_id = BlockId::from(convert_block_number_to_block_number_or_tag(block).unwrap());
        let trace_opt = self.reth_trace.trace_block(block_id).await?;

        let trace = trace_opt.ok_or(RethMiddlewareError::MissingTrace)?;

        let ethers_trace: Vec<EthersTrace> =
            trace.into_iter().map(|trace| reth_trace_to_ethers(trace)).collect();

        Ok(ethers_trace)
    }
}

// thinking of implementing a request so we don't have to change the anvil fork.rs but adds useless
// indirection

/*
impl<M> RethMiddleware<M>
where
    Self: EthApiServer + EthApiSpec + 'static,
    M: Middleware,

{
    pub async fn request<T, R>(&self, method: &str, params: T) -> Result<R, RethMiddlewareError<M>>
    where
        T: Debug + Serialize + Send + Sync,
        R: Serialize + DeserializeOwned + Debug + Send,

    {
        match method {
            "eth_call" => {
                let tx = serde_json::from_value::<&TypedTransaction>(params);
                let block = serde_json::from_value::<Option<EthersBlockId>>(params);
                let res = <&RethMiddleware<M> as Middleware>::call(&self, tx, block).await?;
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
            _ => ProviderError::from(ProviderError::UnsupportedRPC(method.to_string
    }
}


*/
