// Numerous acts of type terrorism having been commited during the making of this program. Please forgive us.

use core::option;
use eyre::Result;
use reth_db::transaction;
use std::{str::FromStr, error::Error};

use ethers::types::{
    transaction::{
        eip2718::TypedTransaction as EthersTypedTransaction,
        eip2930::{
            AccessList as EthersAccessList, AccessListItem as EthersAccessListItem,
            AccessListWithGasUsed as EthersAccessListWithGasUsed,
        },
    },
    Address as EthersAddress, BlockId as EthersBlockId, BlockNumber as EthersBlockNumber,
    Bloom as EthersBloom, Filter as EthersFilter, FilterBlockOption as EthersFilterBlockOption,
    Log as EthersLog, NameOrAddress as EthersNameOrAddress, OtherFields, Topic as EthersTopic,
    Transaction as EthersTransaction, TransactionReceipt as EthersTransactionReceipt,
    ValueOrArray as EthersValueOrArray, H256 as EthersH256, U256 as EthersU256, U64 as EthersU64,
    EIP1186ProofResponse as EthersEIP1186ProofResponse,
    FeeHistory as EthersFeeHistory, Bytes as EthersBytes, StorageProof as EthersStorageProof,
    Block as EthersBlock, Withdrawal as EthersWithdrawal, Trace
};

use reth_primitives::{
    serde_helper::JsonStorageKey, AccessList, AccessListItem, AccessListWithGasUsed, Address,
    BlockHash, BlockId, BlockNumberOrTag, Bloom, Bytes, H160, H256, U128, U256, U64, U8, Withdrawal,
};

use reth_rpc_types::{
    CallRequest, Filter, FilterBlockOption, Log, Topic, TransactionReceipt, ValueOrArray, Transaction, FeeHistory, StorageProof, Rich, Block, RichBlock, BlockTransactions::{Full, self}
};

use reth_revm::primitives::ruint::Uint;
use reth_rpc_types::EIP1186AccountProofResponse;
reth::

pub trait ToEthers<T> {
    /// Reth -> Ethers
    fn into_ethers(self) -> T;
}

pub trait ToReth<T> {
    /// Reth -> Ethers
    fn into_reth(self) -> T;
}

impl ToEthers<EthersU64> for U256 {
    fn into_ethers(self) -> EthersU64 {
        self.to_le_bytes().into()
    }
}

impl ToEthers<EthersU64> for U8 {
    fn into_ethers(self) -> EthersU64 {
        self.to_le_bytes().into()
    }
}

impl ToEthers<EthersU256> for U128 {
    fn into_ethers(self) -> EthersU256 {
        self.to_le_bytes().into()
    }
}

impl ToEthers<EthersH256> for H256 {
    fn into_ethers(self) -> EthersH256 {
        self.into()
    }
}

impl ToEthers<EthersU256> for U64 {
    fn into_ethers(self) -> EthersU256 {
        self.as_u64().into()
    }
}

impl ToEthers<EthersBloom> for Bloom {
    fn into_ethers(self) -> EthersBloom {
        self.to_fixed_bytes().into()
    }
}

impl ToEthers<EthersBytes> for Bytes {
    fn into_ethers(self) -> EthersBytes {
        self.to_vec().into()
    }
}

impl ToEthers<EthersLog> for Log {
    fn into_ethers(self) -> EthersLog {
        EthersLog {
            address: self.address.into(),
            topics: self.topics.into_iter().map(|topic| topic.into()).collect(),
            data: self.data.to_vec().into(),
            block_hash: self.block_hash.map(|hash| hash.into()),
            block_number: self.block_number.map(|num| num.to_le_bytes().into()),
            transaction_hash: self.transaction_hash.map(|hash| hash.into()),
            transaction_index: self.transaction_index.map(|idx| idx.to_le_bytes().into()),
            log_index: self.log_index.map(|idx| idx.into()),
            transaction_log_index: todo!(),
            log_type: todo!(),
            removed: Some(self.removed),
        }
    }
}

impl ToEthers<EthersTransaction> for Transaction {
    fn into_ethers(self) -> EthersTransaction {
        EthersTransaction {
            hash: self.hash.into(),
            nonce: self.nonce.into(),
            block_hash: self.block_hash.map(|hash| hash.into()),
            block_number: self.block_number.map(|n| n.into_ethers()),
            transaction_index: self.transaction_index.map(|n| n.into_ethers()),
            from: self.from.into(),
            to: self.to.map(|t| t.into()),
            value: self.value.into(),
            gas_price: self.gas_price.map(|p| p.into_ethers()),
            gas: self.gas.into(),
            input: self.input.to_vec().into(),
            v: self.signature.clone().map_or(0.into(), |sig| sig.v.into_ethers()),
            r: self.signature.clone().map_or(0.into(), |sig| sig.r.into()),
            s: self.signature.clone().map_or(0.into(), |sig| sig.s.into()),
            transaction_type: self.transaction_type,
            access_list: self.access_list.map(|list| list.into()),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas.map(|p| p.into_ethers()),
            max_fee_per_gas: self.max_fee_per_gas.map(|p| p.into_ethers()),
            chain_id: self.chain_id.map(|id| id.into_ethers()),
            ..Default::default()
        }
    }
}



pub fn ethers_block_id_to_reth_block_id(block_id: EthersBlockId) -> BlockId {
    match block_id {
        EthersBlockId::Hash(hash) => BlockId::Hash(BlockHash::from_slice(hash.as_bytes()).into()),
        EthersBlockId::Number(number) => {
            BlockId::Number(BlockNumberOrTag::Number(number.as_number().unwrap().as_u64()))
        }
    }
}

//Access List Conversion
pub fn ethers_access_list_to_reth_access_list(access_list: EthersAccessList) -> AccessList {
    AccessList(
        access_list
            .0
            .into_iter()
            .map(|item| AccessListItem {
                address: Address::from_slice(item.address.as_bytes()),
                storage_keys: item
                    .storage_keys
                    .into_iter()
                    .map(|key| H256::from_slice(key.as_bytes()))
                    .collect(),
            })
            .collect(),
    )
}

pub fn reth_access_list_to_ethers_access_list(access_list: AccessList) -> EthersAccessList {
    EthersAccessList(
        access_list
            .0
            .into_iter()
            .map(|item| EthersAccessListItem {
                address: EthersAddress::from_slice(item.address.as_bytes()),
                storage_keys: item
                    .storage_keys
                    .into_iter()
                    .map(|key| EthersH256::from_slice(key.as_bytes()))
                    .collect(),
            })
            .collect(),
    )
}

pub fn opt_reth_access_list_to_ethers_access_list(
    opt_access_list: Option<Vec<AccessListItem>>,
) -> EthersAccessList {
    let access_list = opt_access_list.unwrap_or_else(Vec::new);
    EthersAccessList(
        access_list
            .into_iter()
            .map(|item| EthersAccessListItem {
                address: EthersAddress::from_slice(item.address.as_bytes()),
                storage_keys: item
                    .storage_keys
                    .into_iter()
                    .map(|key| EthersH256::from_slice(key.as_bytes()))
                    .collect(),
            })
            .collect(),
    )
}

pub fn reth_access_list_with_gas_used_to_ethers(
    access_list_with_gas_used: AccessListWithGasUsed,
) -> EthersAccessListWithGasUsed {
    EthersAccessListWithGasUsed {
        access_list: EthersAccessList(
            access_list_with_gas_used
                .access_list
                .0
                .into_iter()
                .map(|item| EthersAccessListItem {
                    address: ethers::types::Address::from_slice(item.address.as_bytes()),
                    storage_keys: item
                        .storage_keys
                        .into_iter()
                        .map(|key| ethers::types::H256::from_slice(key.as_bytes()))
                        .collect(),
                })
                .collect(),
        ),
        gas_used: access_list_with_gas_used.gas_used.into(),
    }
}

// TODO: implement Name(String)
pub fn ethers_typed_transaction_to_reth_call_request(tx: &EthersTypedTransaction) -> CallRequest {
    let name_or_addr = tx.to().map(|addr| match addr {
        EthersNameOrAddress::Name(_) => None,
        EthersNameOrAddress::Address(a) => Some(Into::<H160>::into(*a)),
    });

    let mut into_tx = CallRequest {
        from: tx.from().map(|addr| Into::<H160>::into(*addr)),
        to: name_or_addr.map(|a| a.unwrap()),
        gas_price: tx.gas_price().map(|gas| gas.into()),
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
        gas: tx.gas().map(|gas| Into::<U256>::into(*gas)),
        value: tx.value().map(|val| Into::<U256>::into(*val)),
        data: tx.data().map(|d| Into::<Bytes>::into(d.to_vec())),
        nonce: tx.nonce().map(|n| Into::<U256>::into(*n)),
        chain_id: tx.chain_id().map(|id| id.into()),
        access_list: tx
            .access_list()
            .map(|list| ethers_access_list_to_reth_access_list(list.clone())),
        transaction_type: None,
    };

    match tx {
        EthersTypedTransaction::Legacy(_) => {
            into_tx.transaction_type = Some(Uint::from(0));
            ()
        }
        EthersTypedTransaction::Eip2930(inner_tx) => {
            into_tx.transaction_type = Some(Uint::from(1));
            ()
        }
        EthersTypedTransaction::Eip1559(inner_tx) => {
            into_tx.max_fee_per_gas = inner_tx.max_fee_per_gas.map(|gas| gas.into());
            into_tx.max_priority_fee_per_gas =
                inner_tx.max_priority_fee_per_gas.map(|gas| gas.into());
            into_tx.transaction_type = Some(Uint::from(2));
            ()
        }
    }

    into_tx
}

pub fn reth_rpc_transaction_to_ethers(reth_tx: Transaction) -> EthersTransaction {
    EthersTransaction {
        hash: reth_tx.hash.into(),
        nonce: reth_tx.nonce.into(),
        block_hash: reth_tx.block_hash.map(|hash| hash.into()),
        block_number: reth_tx.block_number.map(|n| n.into_ethers()),
        transaction_index: reth_tx.transaction_index.map(|n| n.into_ethers()),
        from: reth_tx.from.into(),
        to: reth_tx.to.map(|t| t.into()),
        value: reth_tx.value.into(),
        gas_price: reth_tx.gas_price.map(|p| p.into_ethers()),
        gas: reth_tx.gas.into(),
        input: reth_tx.input.to_vec().into(),
        v: reth_tx.signature.clone().map_or(0.into(), |sig| sig.v.into_ethers()),
        r: reth_tx.signature.clone().map_or(0.into(), |sig| sig.r.into()),
        s: reth_tx.signature.clone().map_or(0.into(), |sig| sig.s.into()),
        transaction_type: reth_tx.transaction_type,
        access_list: reth_tx.access_list.map(|list| list.into()),
        max_priority_fee_per_gas: reth_tx.max_priority_fee_per_gas.map(|p| p.into_ethers()),
        max_fee_per_gas: reth_tx.max_fee_per_gas.map(|p| p.into_ethers()),
        chain_id: reth_tx.chain_id.map(|id| id.into_ethers()),
        ..Default::default()
    }
}

fn convert_block_number_to_block_number_or_tag(
    block: EthersBlockNumber,
) -> Result<BlockNumberOrTag> {
    match block {
        ethers::types::BlockNumber::Latest => Ok(BlockNumberOrTag::Latest),
        ethers::types::BlockNumber::Finalized => Ok(BlockNumberOrTag::Finalized),
        ethers::types::BlockNumber::Safe => Ok(BlockNumberOrTag::Safe),
        ethers::types::BlockNumber::Earliest => Ok(BlockNumberOrTag::Earliest),
        ethers::types::BlockNumber::Pending => Ok(BlockNumberOrTag::Pending),
        ethers::types::BlockNumber::Number(n) => Ok(BlockNumberOrTag::Number(n.as_u64())),
    }
}

fn convert_topics(topics: &[Option<EthersTopic>; 4]) -> [Option<Topic>; 4] {
    let mut new_topics: Vec<Option<Topic>> = Vec::new();

    for (i, topic) in topics.into_iter().enumerate() {
        new_topics[i] = topic.as_ref().map(&option_convert_valueORarray).clone();
    }

    new_topics.try_into().unwrap()
}

/// ---------------------------

// need to generalize the following 2 functions

fn option_convert_valueORarray<T, U>(val: &EthersValueOrArray<Option<T>>) -> ValueOrArray<Option<U>>
where
    T: Clone,
    U: From<T>,
{
    match val {
        EthersValueOrArray::Value(Some(addr)) => ValueOrArray::Value(Some(addr.clone().into())),
        EthersValueOrArray::Value(None) => ValueOrArray::Value(None),
        EthersValueOrArray::Array(addrs) => {
            ValueOrArray::Array(addrs.into_iter().map(|a| a.clone().map(U::from)).collect())
        }
    }
}

fn convert_valueORarray<T, U>(val: &EthersValueOrArray<T>) -> ValueOrArray<U>
where
    T: Clone,
    U: From<T>,
{
    match val {
        EthersValueOrArray::Value(addr) => ValueOrArray::Value(addr.clone().into()),
        EthersValueOrArray::Array(addrs) => {
            ValueOrArray::Array(addrs.into_iter().map(|a| Into::<U>::into(a.clone())).collect())
        }
    }
}

/// ---------------------------

pub fn ethers_filter_to_reth_filter(filter: &EthersFilter) -> Filter {
    return Filter {
        block_option: match &filter.block_option {
            EthersFilterBlockOption::AtBlockHash(x) => {
                FilterBlockOption::AtBlockHash(Into::<H256>::into(*x))
            }
            EthersFilterBlockOption::Range { from_block, to_block } => FilterBlockOption::Range {
                from_block: convert_block_number_to_block_number_or_tag(from_block.unwrap()).ok(),
                to_block: convert_block_number_to_block_number_or_tag(to_block.unwrap()).ok(),
            },
        },

        address: match &filter.address {
            Some(addr) => Some(convert_valueORarray(&addr)),
            None => None,
        },

        topics: convert_topics(&filter.topics),
    }
}

pub fn reth_rpc_log_to_ethers(log: Log) -> EthersLog {
    EthersLog {
        address: log.address.into(),
        topics: log.topics.into_iter().map(|topic| topic.into()).collect(),
        data: log.data.to_vec().into(),
        block_hash: log.block_hash.map(|hash| hash.into()),
        block_number: log.block_number.map(|num| num.to_le_bytes().into()),
        transaction_hash: log.transaction_hash.map(|hash| hash.into()),
        transaction_index: log.transaction_index.map(|idx| idx.to_le_bytes().into()),
        log_index: log.log_index.map(|idx| idx.into()),
        transaction_log_index: todo!(),
        log_type: todo!(),
        removed: Some(log.removed),
    }
}

pub fn reth_transaction_receipt_to_ethers(receipt: TransactionReceipt) -> EthersTransactionReceipt {
    EthersTransactionReceipt {
        transaction_hash: receipt.transaction_hash.unwrap().into(),
        transaction_index: receipt.transaction_index.unwrap().into_ethers(),
        block_hash: receipt.block_hash.map(|hash| hash.into()),
        block_number: receipt.block_number.map(|num| num.into_ethers()),
        from: receipt.from.into(),
        to: receipt.to.map(|t| t.into()),
        cumulative_gas_used: receipt.cumulative_gas_used.into(),
        gas_used: receipt.gas_used.map(|gas| gas.into()),
        contract_address: receipt.contract_address.map(|addr| addr.into()),
        logs: receipt.logs.into_iter().map(|log| log.into_ethers()).collect(),
        status: receipt.status_code.map(|num| num.as_u64().into()),
        root: receipt.state_root.map(|root| root.into()),
        logs_bloom: receipt.logs_bloom.into_ethers(),
        transaction_type: Some(receipt.transaction_type.into_ethers()),
        effective_gas_price: Some(U256::from(receipt.effective_gas_price).into()),
        other: OtherFields::default(),
    }
}

pub fn reth_storage_proof_to_ethers(proof: StorageProof) -> EthersStorageProof {
    EthersStorageProof {
        key: H256::from_str(&Into::<String>::into(proof.key)).unwrap().into(),
        proof: proof.proof.into_iter().map(|p| p.into_ethers()).collect(),
        value: proof.value.into(),
    }
}

pub fn reth_proof_to_ethers(proof: EIP1186AccountProofResponse) -> EthersEIP1186ProofResponse {
    EthersEIP1186ProofResponse {
        address: proof.address.into(),
        balance: proof.balance.into(),
        code_hash: proof.code_hash.into(),
        nonce: proof.nonce.into(),
        storage_hash: proof.storage_hash.into(),
        account_proof: proof.account_proof.into_iter().map(|a| a.into_ethers()).collect(),
        storage_proof: proof
            .storage_proof
            .into_iter()
            .map(|s| reth_storage_proof_to_ethers(s))
            .collect(),
    }
}

pub fn reth_fee_history_to_ethers(fee_history: FeeHistory) -> EthersFeeHistory {
    EthersFeeHistory {
        base_fee_per_gas: fee_history.base_fee_per_gas.into_iter().map(|fee| fee.into()).collect(),
        gas_used_ratio: fee_history.gas_used_ratio.into_iter().map(|gas| gas).collect(),
        oldest_block: fee_history.oldest_block.into(),
        reward: fee_history
            .reward
            .unwrap()
            .into_iter()
            .map(|inner_vec| inner_vec.into_iter().map(|num| num.into()).collect())
            .collect(),
    }
}


pub fn reth_richblock_txs_into_ethers<R, E>(txs: Vec<R>) -> Vec<E> 
where
    R: ToEthers<E>
{
    txs.into_iter().map(|t: R| t.into_ethers()).collect::<Vec<E>>()
}

pub fn reth_withraw_to_ethers(withdraw: Withdrawal) -> EthersWithdrawal {
    
    EthersWithdrawal {
        index: withdraw.index.into(),
        validator_index: withdraw.index.into(),
        address: withdraw.address.into(),
        amount: withdraw.amount.into(),
    }
}



pub fn full_rich_block_to_ethers(rich_block: RichBlock) -> EthersBlock<EthersTransaction> {

    let block = rich_block.inner;
    let header = block.header;

    let txs: Vec<Transaction>;
    if let BlockTransactions::Full(transactions) = block.transactions {
        txs = transactions
    };

    EthersBlock {
        hash: header.hash.map(|h| h.into()),
        parent_hash: header.parent_hash.into(),
        uncles_hash: header.uncles_hash.into(),
        author: Some(header.miner.into()),
        state_root: header.state_root.into(),
        transactions_root: header.transactions_root.into(),
        receipts_root: header.receipts_root.into(),
        number: header.number.map(|num| num.into_ethers()),
        gas_used: header.gas_used.into(),
        gas_limit: header.gas_limit.into(),
        extra_data: header.extra_data.to_vec().into(),
        logs_bloom: Some(header.logs_bloom.into_ethers()),
        timestamp: header.timestamp.into(),
        difficulty: header.difficulty.into(),
        total_difficulty: block.total_difficulty.map(|d| d.into()),
        seal_fields: vec![], // TODO
        uncles: block.uncles.into_iter().map(|unc| unc.into()).collect(),
        transactions: reth_richblock_txs_into_ethers::<Transaction, EthersTransaction>(txs), 
        size: block.size.map(|s| s.into()),
        mix_hash: Some(header.mix_hash.into()),
        nonce: header.nonce.into(),
        base_fee_per_gas: header.base_fee_per_gas.map(|fee| fee.into()),
        withdrawals_root: header.withdrawals_root.map(|root| root.into()),
        withdrawals: block.withdrawals.map(|list| list.into_iter().map(|w| reth_withraw_to_ethers(w)).collect()),
        other: OtherFields::default(),
    }
}


pub fn hash_rich_block_to_ethers(rich_block: RichBlock) -> EthersBlock<EthersH256> {

    let block = rich_block.inner;
    let header = block.header;

    let tx_hashes: Vec<H256>;
    if let BlockTransactions::Hashes(hashes) = block.transactions {
        tx_hashes = hashes
    };

    EthersBlock {
        hash: header.hash.map(|h| h.into()),
        parent_hash: header.parent_hash.into(),
        uncles_hash: header.uncles_hash.into(),
        author: Some(header.miner.into()),
        state_root: header.state_root.into(),
        transactions_root: header.transactions_root.into(),
        receipts_root: header.receipts_root.into(),
        number: header.number.map(|num| num.into_ethers()),
        gas_used: header.gas_used.into(),
        gas_limit: header.gas_limit.into(),
        extra_data: header.extra_data.to_vec().into(),
        logs_bloom: Some(header.logs_bloom.into_ethers()),
        timestamp: header.timestamp.into(),
        difficulty: header.difficulty.into(),
        total_difficulty: block.total_difficulty.map(|d| d.into()),
        seal_fields:  vec![], // TODO
        uncles: block.uncles.into_iter().map(|unc| unc.into()).collect(),
        transactions: reth_richblock_txs_into_ethers::<H256, EthersH256>(tx_hashes), 
        size: block.size.map(|s| s.into()),
        mix_hash: Some(header.mix_hash.into()),
        nonce: header.nonce.into(),
        base_fee_per_gas: header.base_fee_per_gas.map(|fee| fee.into()),
        withdrawals_root: header.withdrawals_root.map(|root| root.into()),
        withdrawals: block.withdrawals.map(|list| list.into_iter().map(|w| reth_withraw_to_ethers(w)).collect()),
        other: OtherFields::default(),
    }
}

pub fn convert_location_to_json_key(location: EthersH256) -> JsonStorageKey {
    let location = location.to_fixed_bytes();
    let location_u256: U256 = U256::from_be_bytes(location);
    JsonStorageKey::from(location_u256)
}

pub fn convert_Ethers_U256_to_Reth_U64(u256: EthersU256) -> U64 {
    let u256 = u256.as_u64();
    u256.into()
}

pub fn convert_Reth_U256_to_Ethers_U64(u256: U256) -> EthersU64 {
    let u256: EthersU256 = u256.into();
    let u256 = u256.as_u64();
    u256.into()
}

pub fn convert_Reth_U64_to_Ethers_U256(u64: U64) -> EthersU256 {
    let u64t = u64.as_u64();
    u64t.into()
}

pub fn reth_trace_to_ethers(RethTrace: Trace) -> EthersTrace {
    todo!()
}
