use crate::{utils::*, RethMiddleware, RethMiddlewareError};
use async_trait::async_trait;
use ethers::{
    providers::{ProviderError, Middleware},
    types::{transaction::eip2718::TypedTransaction, BlockId, Bytes},
};
use reth_network_api::NetworkInfo;
use reth_provider::{BlockProvider, EvmEnvProvider, StateProviderFactory, BlockProviderIdExt, BlockIdProvider, HeaderProvider};
use reth_rpc::{eth::{EthApi, EthTransactions, *}, EthApiSpec};
use reth_rpc_api::EthApiServer;
use reth_transaction_pool::TransactionPool;

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
            .estimate_gas(call_request, block_id, None)
            .await?
            .0
            .into())
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


    /// Get the block number
    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        self.inner().get_block_number().await.map_err(MiddlewareError::from_err)
    }

    /// Gets the block at `block_hash_or_number` (transaction hashes only)
    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        self.inner().get_block(block_hash_or_number).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the block at `block_hash_or_number` (full transactions included)
    async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<Option<Block<Transaction>>, Self::Error> {
        self.inner()
            .get_block_with_txs(block_hash_or_number)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Gets the block uncle count at `block_hash_or_number`
    async fn get_uncle_count<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
    ) -> Result<U256, Self::Error> {
        self.inner().get_uncle_count(block_hash_or_number).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the block uncle at `block_hash_or_number` and `idx`
    async fn get_uncle<T: Into<BlockId> + Send + Sync>(
        &self,
        block_hash_or_number: T,
        idx: U64,
    ) -> Result<Option<Block<H256>>, Self::Error> {
        self.inner().get_uncle(block_hash_or_number, idx).await.map_err(MiddlewareError::from_err)
    }

    /// Returns the nonce of the address
    async fn get_transaction_count<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_transaction_count(from, block).await.map_err(MiddlewareError::from_err)
    }

    /// Sends a transaction to a single Ethereum node and return the estimated amount of gas
    /// required (as a U256) to send it This is free, but only an estimate. Providing too little
    /// gas will result in a transaction being rejected (while still consuming all provided
    /// gas).
    async fn estimate_gas(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().estimate_gas(tx, block).await.map_err(MiddlewareError::from_err)
    }

    /// Sends the read-only (constant) transaction to a single Ethereum node and return the result
    /// (as bytes) of executing it. This is free, since it does not change any state on the
    /// blockchain.
    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().call(tx, block).await.map_err(MiddlewareError::from_err)
    }

    /// Return current client syncing status. If IsFalse sync is over.
    async fn syncing(&self) -> Result<SyncingStatus, Self::Error> {
        self.inner().syncing().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the currently configured chain id, a value used in replay-protected
    /// transaction signing as introduced by EIP-155.
    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        self.inner().get_chainid().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the network version.
    async fn get_net_version(&self) -> Result<String, Self::Error> {
        self.inner().get_net_version().await.map_err(MiddlewareError::from_err)
    }

    /// Returns the account's balance
    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        self.inner().get_balance(from, block).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the transaction with `transaction_hash`
    async fn get_transaction<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<Transaction>, Self::Error> {
        self.inner().get_transaction(transaction_hash).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the transaction receipt with `transaction_hash`
    async fn get_transaction_receipt<T: Send + Sync + Into<TxHash>>(
        &self,
        transaction_hash: T,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        self.inner()
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Returns all receipts for a block.
    ///
    /// Note that this uses the `eth_getBlockReceipts` RPC, which is
    /// non-standard and currently supported by Erigon.
    async fn get_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner().get_block_receipts(block).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the current gas price as estimated by the node
    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        self.inner().get_gas_price().await.map_err(MiddlewareError::from_err)
    }

    /// Gets a heuristic recommendation of max fee per gas and max priority fee per gas for
    /// EIP-1559 compatible transactions.
    async fn estimate_eip1559_fees(
        &self,
        estimator: Option<fn(U256, Vec<Vec<U256>>) -> (U256, U256)>,
    ) -> Result<(U256, U256), Self::Error> {
        self.inner().estimate_eip1559_fees(estimator).await.map_err(MiddlewareError::from_err)
    }

    /// Gets the accounts on the node
    async fn get_accounts(&self) -> Result<Vec<Address>, Self::Error> {
        self.inner().get_accounts().await.map_err(MiddlewareError::from_err)
    }

    /// Send the raw RLP encoded transaction to the entire Ethereum network and returns the
    /// transaction's hash This will consume gas from the account that signed the transaction.
    async fn send_raw_transaction<'a>(
        &'a self,
        tx: Bytes,
    ) -> Result<PendingTransaction<'a, Self::Provider>, Self::Error> {
        self.inner().send_raw_transaction(tx).await.map_err(MiddlewareError::from_err)
    }

    /// This returns true if either the middleware stack contains a `SignerMiddleware`, or the
    /// JSON-RPC provider has an unlocked key that can sign using the `eth_sign` call. If none of
    /// the above conditions are met, then the middleware stack is not capable of signing data.
    async fn is_signer(&self) -> bool {
        self.inner().is_signer().await
    }

    /// Signs data using a specific account. This account needs to be unlocked,
    /// or the middleware stack must contain a `SignerMiddleware`
    async fn sign<T: Into<Bytes> + Send + Sync>(
        &self,
        data: T,
        from: &Address,
    ) -> Result<Signature, Self::Error> {
        self.inner().sign(data, from).await.map_err(MiddlewareError::from_err)
    }

    

    ////// Contract state

    /// Returns an array (possibly empty) of logs that match the filter
    async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, Self::Error> {
        self.inner().get_logs(filter).await.map_err(MiddlewareError::from_err)
    }

    /// Returns a stream of logs are loaded in pages of given page size
    fn get_logs_paginated<'a>(
        &'a self,
        filter: &Filter,
        page_size: u64,
    ) -> LogQuery<'a, Self::Provider> {
        self.inner().get_logs_paginated(filter, pa
    


    /// Returns the deployed code at a given address
    async fn get_code<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        at: T,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        self.inner().get_code(at, block).await.map_err(MiddlewareError::from_err)
    }

    
    /// Returns the EIP-1186 proof response
    /// <https://github.com/ethereum/EIPs/issues/1186>
    async fn get_proof<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        locations: Vec<H256>,
        block: Option<BlockId>,
    ) -> Result<EIP1186ProofResponse, Self::Error> {
        self.inner().get_proof(from, locations, block).await.map_err(MiddlewareError::from_err)
    }


    /// After replaying any previous transactions in the same block,
    /// Replays a transaction, returning the traces configured with passed options
    async fn debug_trace_transaction(
        &self,
        tx_hash: TxHash,
        trace_options: GethDebugTracingOptions,
    ) -> Result<GethTrace, Self::Error> {
        self.inner()
            .debug_trace_transaction(tx_hash, trace_options)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Executes the given call and returns a number of possible traces for it
    async fn debug_trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: T,
        block: Option<BlockId>,
        trace_options: GethDebugTracingCallOptions,
    ) -> Result<GethTrace, Self::Error> {
        self.inner()
            .debug_trace_call(req, block, trace_options)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Replays all transactions in a given block (specified by block number) and returns the traces
    /// configured with passed options
    /// Ref:
    /// [Here](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-debug#debugtraceblockbynumber)
    async fn debug_trace_block_by_number(
        &self,
        block: Option<BlockNumber>,
        trace_options: GethDebugTracingOptions,
    ) -> Result<Vec<GethTrace>, Self::Error> {
        self.inner()
            .debug_trace_block_by_number(block, trace_options)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Replays all transactions in a given block (specified by block hash) and returns the traces
    /// configured with passed options
    /// Ref:
    /// [Here](https://geth.ethereum.org/docs/interacting-with-geth/rpc/ns-debug#debugtraceblockbyhash)
    async fn debug_trace_block_by_hash(
        &self,
        block: H256,
        trace_options: GethDebugTracingOptions,
    ) -> Result<Vec<GethTrace>, Self::Error> {
        self.inner()
            .debug_trace_block_by_hash(block, trace_options)
            .await
            .map_err(MiddlewareError::from_err)
    }

    // Parity `trace` support

    /// Executes the given call and returns a number of possible traces for it
    async fn trace_call<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: T,
        trace_type: Vec<TraceType>,
        block: Option<BlockNumber>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner().trace_call(req, trace_type, block).await.map_err(MiddlewareError::from_err)
    }

    /// Executes given calls and returns a number of possible traces for each
    /// call
    async fn trace_call_many<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        req: Vec<(T, Vec<TraceType>)>,
        block: Option<BlockNumber>,
    ) -> Result<Vec<BlockTrace>, Self::Error> {
        self.inner().trace_call_many(req, block).await.map_err(MiddlewareError::from_err)
    }

    /// Traces a call to `eth_sendRawTransaction` without making the call, returning the traces
    async fn trace_raw_transaction(
        &self,
        data: Bytes,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner()
            .trace_raw_transaction(data, trace_type)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Replays a transaction, returning the traces
    async fn trace_replay_transaction(
        &self,
        hash: H256,
        trace_type: Vec<TraceType>,
    ) -> Result<BlockTrace, Self::Error> {
        self.inner()
            .trace_replay_transaction(hash, trace_type)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Replays all transactions in a block returning the requested traces for each transaction
    async fn trace_replay_block_transactions(
        &self,
        block: BlockNumber,
        trace_type: Vec<TraceType>,
    ) -> Result<Vec<BlockTrace>, Self::Error> {
        self.inner()
            .trace_replay_block_transactions(block, trace_type)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Returns traces created at given block
    async fn trace_block(&self, block: BlockNumber) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_block(block).await.map_err(MiddlewareError::from_err)
    }

    /// Return traces matching the given filter
    async fn trace_filter(&self, filter: TraceFilter) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_filter(filter).await.map_err(MiddlewareError::from_err)
    }

    /// Returns trace at the given position
    async fn trace_get<T: Into<U64> + Send + Sync>(
        &self,
        hash: H256,
        index: Vec<T>,
    ) -> Result<Trace, Self::Error> {
        self.inner().trace_get(hash, index).await.map_err(MiddlewareError::from_err)
    }

    /// Returns all traces of a given transaction
    async fn trace_transaction(&self, hash: H256) -> Result<Vec<Trace>, Self::Error> {
        self.inner().trace_transaction(hash).await.map_err(MiddlewareError::from_err)
    }

    // Parity namespace

    /// Returns all receipts for that block. Must be done on a parity node.
    async fn parity_block_receipts<T: Into<BlockNumber> + Send + Sync>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, Self::Error> {
        self.inner().parity_block_receipts(block).await.map_err(MiddlewareError::from_err)
    }

    /// Create a new subscription
    ///
    /// This method is hidden as subscription lifecycles are intended to be
    /// handled by a [`SubscriptionStream`] object.
    #[doc(hidden)]
    async fn subscribe<T, R>(
        &self,
        params: T,
    ) -> Result<SubscriptionStream<'_, Self::Provider, R>, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send + Sync,
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe(params).await.map_err(MiddlewareError::from_err)
    }

    /// Instruct the RPC to cancel a subscription by its ID
    ///
    /// This method is hidden as subscription lifecycles are intended to be
    /// handled by a [`SubscriptionStream`] object
    #[doc(hidden)]
    async fn unsubscribe<T>(&self, id: T) -> Result<bool, Self::Error>
    where
        T: Into<U256> + Send + Sync,
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().unsubscribe(id).await.map_err(MiddlewareError::from_err)
    }

    /// Subscribe to a stream of incoming blocks.
    ///
    /// This function is only available on pubsub clients, such as Websockets
    /// or IPC. For a polling alternative available over HTTP, use
    /// [`Middleware::watch_blocks`]. However, be aware that polling increases
    /// RPC usage drastically.
    async fn subscribe_blocks(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, Block<TxHash>>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_blocks().await.map_err(MiddlewareError::from_err)
    }

    /// Subscribe to a stream of pending transactions.
    ///
    /// This function is only available on pubsub clients, such as Websockets
    /// or IPC. For a polling alternative available over HTTP, use
    /// [`Middleware::watch_pending_transactions`]. However, be aware that
    /// polling increases RPC usage drastically.
    async fn subscribe_pending_txs(
        &self,
    ) -> Result<SubscriptionStream<'_, Self::Provider, TxHash>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_pending_txs().await.map_err(MiddlewareError::from_err)
    }

    /// Subscribe to a stream of event logs matchin the provided [`Filter`].
    ///
    /// This function is only available on pubsub clients, such as Websockets
    /// or IPC. For a polling alternative available over HTTP, use
    /// [`Middleware::watch`]. However, be aware that polling increases
    /// RPC usage drastically.
    async fn subscribe_logs<'a>(
        &'a self,
        filter: &Filter,
    ) -> Result<SubscriptionStream<'a, Self::Provider, Log>, Self::Error>
    where
        <Self as Middleware>::Provider: PubsubClient,
    {
        self.inner().subscribe_logs(filter).await.map_err(MiddlewareError::from_err)
    }


    
    async fn fee_history<T: Into<U256> + serde::Serialize + Send + Sync>(
        &self,
        block_count: T,
        last_block: BlockNumber,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory, Self::Error> {
        self.inner()
            .fee_history(block_count, last_block, reward_percentiles)
            .await
            .map_err(MiddlewareError::from_err)
    }

    /// Querty the node for an EIP-2930 Access List.
    ///
    /// See the
    /// [EIP-2930 documentation](https://eips.ethereum.org/EIPS/eip-2930) for
    /// details
    async fn create_access_list(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<AccessListWithGasUsed, Self::Error> {
        self.inner().create_access_list(tx, block).await.map_err(MiddlewareError::from_err)
        }

}



    

impl<M, Client, Pool, Network> RethMiddleware<M, Client, Pool, Network>
    where
        M: Middleware,
        Client: BlockProvider + EvmEnvProvider + StateProviderFactory + 'static,
        Pool: TransactionPool + Clone + 'static,
        Network: NetworkInfo + 'static,
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
                let res = self.reth_api().call(tx, block).await?;
                Ok(res)
            }
            "eth_estimateGas" => {
                let tx = serde_json::from_value::<&TypedTransaction>(params)?;
                let block = serde_json::from_value::<Option<BlockId>>(params)?;
                let res = self.reth_api.estimate_gas(tx, block).await?;
                Ok(res)
            }
            "eth_createAcessList" => {
                let tx = serde_json::from_value::<&TypedTransaction>(params)?;
                let block = serde_json::from_value::<Option<BlockId>>(params)?;
                let res = self.reth_api.create_access_list(tx, block).await?;
                Ok(res)
            }
            _ => ProviderError::from(ProviderError::UnsupportedRPC(method.to_string())),
        }
    }
}





