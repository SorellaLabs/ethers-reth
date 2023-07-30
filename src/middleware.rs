use crate::{
    type_conversions::{ToEthers, ToReth},
    RethMiddleware, RethMiddlewareError,
};
use async_trait::async_trait;

// Ether rs Types
use ethers::{
    providers::{Middleware, MiddlewareError},
    types::{
        transaction::{
            eip2718::TypedTransaction,
            eip2930::AccessListWithGasUsed as EthersAccessListWithGasUsed,
        },
        Address as EthersAddress, Block as EthersBlock, BlockId as EthersBlockId,
        BlockNumber as EthersBlocKNumber, BlockNumber as EthersBlockNumber,
        BlockTrace as EthersBlockTrace, Bytes as EthersBytes,
        EIP1186ProofResponse as EthersEIP1186ProofResponse, FeeHistory as EthersFeeHistory,
        Filter as EthersFilter, GethDebugTracingCallOptions as EthersDebugTracingCallOptions,
        GethDebugTracingOptions as EthersDebugTracingOptions, GethTrace as EthersGethTrace,
        Log as EthersLog, NameOrAddress, Trace as EthersTrace, TraceType as EthersTraceType,
        Transaction as EthersTransaction, TransactionReceipt as EthersTransactionReceipt,
        TxHash as EthersTxHash, H256 as EthersH256, U256 as EthersU256, U64 as EthersU64,
    },
};

// Reth Types
use reth_primitives::BlockId;
use reth_rpc::eth::revm_utils::EvmOverrides;
use reth_rpc_api::{EthApiServer, EthFilterApiServer};
// use reth_rpc_types::trace::geth::TraceResult;
use reth_rpc_types::{
    trace::{
        common::TraceResult,
        geth::{DefaultFrame, GethTrace},
    },
    Filter,
};

impl<M> RethMiddleware<M>
where
    M: Middleware,
{
    pub async fn get_address<T: Into<NameOrAddress>>(
        &self,
        who: T,
    ) -> Result<EthersAddress, RethMiddlewareError<M>> {
        match who.into() {
            NameOrAddress::Name(ens_name) => {
                self.inner.resolve_name(&ens_name).await.map_err(RethMiddlewareError::from_err)
            }
            NameOrAddress::Address(addr) => Ok(addr),
        }
    }
}

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

    // Call related methods
    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<EthersBlockId>,
    ) -> Result<EthersBytes, Self::Error> {
        let call_request = tx.into_reth();
        let block_id = block.into_reth();

        Ok(self.reth_api.call(call_request, block_id, EvmOverrides::default()).await?.into_ethers())
    }

    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<EthersBlockId>,
    ) -> Result<EthersU256, Self::Error> {
        let call_request = tx.into_reth();
        let block_id = block.into_reth();

        Ok(self.reth_api.estimate_gas(call_request, block_id).await?.into())
    }

    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<EthersBlockId>,
    ) -> Result<EthersAccessListWithGasUsed, Self::Error> {
        let call_request = tx.into_reth();
        let block_id = block.into_reth();

        let result = self.reth_api.create_access_list(call_request, block_id).await?;

        Ok(result.into_ethers())
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
        let index = location.into_reth();
        // convert `block` to `Option<BlockId>`
        let block_id = block.into_reth();

        // call `storage_at` and convert the result
        Ok(self.reth_api.storage_at(from.into(), index, block_id).await?.into())
    }

    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<EthersBlockId>,
    ) -> Result<EthersBytes, Self::Error> {
        let at = self.get_address(at).await?;

        let block_id = block.into_reth();
        let code = self.reth_api.get_code(at.into(), block_id).await?;
        // Convert to EthersBytes
        Ok(code.into_ethers())
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<EthersBlockId>,
    ) -> Result<EthersU256, Self::Error> {
        let from = self.get_address(from).await?;
        Ok(self.reth_api.balance(from.into(), block.into_reth()).await?.into())
    }

    async fn get_proof<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        locations: Vec<EthersH256>,
        block: Option<EthersBlockId>,
    ) -> Result<EthersEIP1186ProofResponse, RethMiddlewareError<M>> {
        let from = self.get_address(from).await?;

        Ok(self
            .reth_api
            .get_proof(from.into(), locations.into_reth(), block.into_reth())
            .await?
            .into_ethers())
    }

    async fn fee_history<T: Into<EthersU256> + Send + Sync>(
        &self,
        block_count: T,
        last_block: EthersBlocKNumber,
        reward_percentiles: &[f64],
    ) -> Result<EthersFeeHistory, Self::Error> {
        Ok(self
            .reth_api
            .fee_history(
                block_count.into().into_reth(),
                last_block.into_reth(),
                Some(reward_percentiles.to_vec()),
            )
            .await?
            .into_ethers())
    }

    // Chain Info

    async fn get_chainid(&self) -> Result<EthersU256, RethMiddlewareError<M>> {
        let chain_id = EthApiServer::chain_id(&self.reth_api)
            .await?
            .ok_or_else(|| RethMiddlewareError::ChainIdUnavailable)?;

        Ok(chain_id.into_ethers())
    }

    async fn get_block_number(&self) -> Result<EthersU64, RethMiddlewareError<M>> {
        Ok(self.reth_api.block_number()?.into_ethers())
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
            Some(reth_tx) => Ok(Some(reth_tx.into_ethers())),
            None => Ok(None),
        }
    }

    async fn get_transaction_receipt<T: Send + Sync + Into<EthersTxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<EthersTransactionReceipt>, RethMiddlewareError<M>> {
        let hash = ethers::types::H256::from_slice(transaction_hash.into().as_bytes());
        match self.reth_api.transaction_receipt(hash.into()).await? {
            Some(receipt) => Ok(Some(receipt.into_ethers())),
            None => Ok(None),
        }
    }

    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<EthersBlockId>,
    ) -> Result<EthersU256, Self::Error> {
        let from = self.get_address(from).await?;

        let block_id = block.into_reth();
        Ok(self.reth_api.transaction_count(from.into(), block_id).await?.into())
    }

    // Blocks

    async fn get_block<T: Into<EthersBlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<EthersBlock<EthersH256>>, Self::Error> {
        let block_id: EthersBlockId = block_hash_or_number.into();

        let block = match block_id {
            EthersBlockId::Hash(hash) => self.reth_api.block_by_hash(hash.into(), false).await?,
            EthersBlockId::Number(num) => {
                self.reth_api.block_by_number(num.into_reth(), false).await?
            }
        };

        Ok(block.into_ethers())
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
                    .uncle_by_block_number_and_index(num.into_reth(), idx.as_usize().into())
                    .await?
            }
        };

        Ok(block.into_ethers())
    }

    async fn get_block_with_txs<T: Into<EthersBlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<EthersBlock<EthersTransaction>>, Self::Error> {
        let block_id = block_hash_or_number.into();

        let block = match block_id {
            EthersBlockId::Hash(hash) => self.reth_api.block_by_hash(hash.into(), true).await?,
            EthersBlockId::Number(num) => {
                self.reth_api.block_by_number(num.into_reth(), true).await?
            }
        };

        Ok(block.into_ethers())
    }

    // Logs

    //TODO: Implement get_logs_paginated
    //TODO: Implement stream event logs (watch)
    //TODO: Watch pending tx

    // Tracing
    async fn trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: T,
        trace_type: Vec<EthersTraceType>,
        block: Option<EthersBlockNumber>,
    ) -> Result<EthersBlockTrace, Self::Error> {
        let tx = req.into();
        let trace = self
            .reth_trace
            .trace_call(tx.into_reth(), trace_type.into_reth(), block.into_reth(), None, None)
            .await?;
        Ok(trace.into_ethers())
    }

    async fn trace_call_many<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: Vec<(T, Vec<EthersTraceType>)>,
        block: Option<EthersBlockNumber>,
    ) -> Result<Vec<EthersBlockTrace>, Self::Error> {
        let tx: Vec<(TypedTransaction, Vec<EthersTraceType>)> =
            req.into_iter().map(|r| (r.0.into(), r.1)).collect();
        Ok(self.reth_trace.trace_call_many(tx.into_reth(), block.into_reth()).await?.into_ethers())
    }

    async fn trace_raw_transaction(
        &self,
        data: EthersBytes,
        trace_type: Vec<EthersTraceType>,
    ) -> Result<EthersBlockTrace, Self::Error> {
        Ok(self
            .reth_trace
            .trace_raw_transaction(data.into_reth(), trace_type.into_reth(), None)
            .await?
            .into_ethers())
    }

    async fn trace_replay_transaction(
        &self,
        hash: EthersH256,
        trace_type: Vec<EthersTraceType>,
    ) -> Result<EthersBlockTrace, Self::Error> {
        Ok(self
            .reth_trace
            .replay_transaction(hash.into(), trace_type.into_reth())
            .await?
            .into_ethers())
    }

    async fn trace_replay_block_transactions(
        &self,
        block: EthersBlockNumber,
        trace_type: Vec<EthersTraceType>,
    ) -> Result<Vec<EthersBlockTrace>, Self::Error> {
        let res = self
            .reth_trace
            .replay_block_transactions(BlockId::Number(block.into_reth()), trace_type.into_reth())
            .await?;
        Ok(res.unwrap().into_ethers())
    }

    async fn trace_block(&self, block: EthersBlockNumber) -> Result<Vec<EthersTrace>, Self::Error> {
        let block_id = block.into_reth();
        let trace_opt = self.reth_trace.trace_block(BlockId::Number(block_id)).await?;
        Ok(trace_opt.ok_or(RethMiddlewareError::MissingTrace)?.into_ethers())
    }

    async fn debug_trace_transaction(
        &self,
        tx_hash: EthersTxHash,
        trace_options: EthersDebugTracingOptions,
    ) -> Result<EthersGethTrace, Self::Error> {
        let debug_trace = self
            .reth_debug
            .debug_trace_transaction(tx_hash.into(), trace_options.into_reth())
            .await?;

        Ok(debug_trace.into_ethers())
    }

    async fn debug_trace_block_by_hash(
        &self,
        block: EthersH256,
        trace_options: EthersDebugTracingOptions,
    ) -> Result<Vec<EthersGethTrace>, Self::Error> {
        let mut debug_trace = self
            .reth_debug
            .debug_trace_block(
                BlockId::from(reth_primitives::H256(block.0)),
                trace_options.into_reth(),
            )
            .await?;

        let mut trace = vec![];

        debug_trace.iter_mut().for_each(|x| {
            trace.push(match x {
                TraceResult::Success { result: val } => val.clone(),
                TraceResult::Error { error: _ } => GethTrace::Default(DefaultFrame::default()),
            });
        });

        Ok(trace.into_ethers())
    }

    async fn debug_trace_block_by_number(
        &self,
        block: Option<ethers::types::BlockNumber>,
        trace_options: EthersDebugTracingOptions,
    ) -> Result<Vec<EthersGethTrace>, Self::Error> {
        let mut debug_trace = self
            .reth_debug
            .debug_trace_block(block.unwrap().into_reth(), trace_options.into_reth())
            .await?;

        let mut trace = vec![];

        debug_trace.iter_mut().for_each(|x| {
            trace.push(match x {
                TraceResult::Success { result: val } => val.clone(),
                TraceResult::Error { error: _ } => GethTrace::Default(DefaultFrame::default()),
            });
        });

        Ok(trace.into_ethers())
    }

    async fn debug_trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        call: T,
        block_id: Option<EthersBlockId>,
        trace_options: EthersDebugTracingCallOptions,
    ) -> Result<EthersGethTrace, Self::Error> {
        let debug_trace = self
            .reth_debug
            .debug_trace_call(
                call.into().into_reth(),
                block_id.into_reth(),
                trace_options.into_reth(),
            )
            .await?;

        Ok(debug_trace.into_ethers())
    }

    async fn trace_get<T: Into<EthersU64> + Send + Sync>(
        &self,
        hash: EthersH256,
        index: Vec<T>,
    ) -> Result<EthersTrace, Self::Error> {
        let index: Vec<usize> = index.into_iter().map(|i| i.into().as_usize()).collect();
        Ok(self.reth_trace.trace_get(hash.into(), index).await?.into_ethers().unwrap())
    }

    async fn trace_transaction(
        &self,
        tx_hash: EthersTxHash,
    ) -> Result<Vec<EthersTrace>, Self::Error> {
        let trace = self.reth_trace.trace_transaction(tx_hash.into()).await?;
        Ok(trace.into_ethers().unwrap())
    }
}
