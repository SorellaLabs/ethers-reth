use std::{ops::Deref, error::Error};

use ethers::types::{
    transaction::{eip2718::TypedTransaction, eip2930::AccessList as EthersAccessList, eip2930::AccessListItem as EthersAccessListItem, eip2930::AccessListWithGasUsed as EthersAccessListWithGasUsed},
    BlockId as EthersBlockId, NameOrAddress,
};
use ethers::types::TransactionReceipt as EthersTransactionReceipt;
use ethers::types::Transaction as EthersTransaction;
use ethers::types::OtherFields;
use ethers::types::Filter as EthersFilter;
use ethers::types::ValueOrArray as EthersValueOrArray;
use ethers::types::Topic as EthersTopic;
use ethers::types::FilterBlockOption as EthersFilterBlockOption;
use ethers::types::BlockNumber as EthersBlockNumber;
use ethers::types::H256 as EthersH256;
use eyre::Result;

use ethers::types::Address as EthersAddress;


use reth_primitives::{
    AccessList, AccessListWithGasUsed, AccessListItem, Address, BlockId, BlockNumberOrTag, Bytes, U256,
    U8, H256, hex::ToHex
};




use reth_rpc_types::TransactionReceipt;
use reth_rpc_types::CallRequest;
use reth_revm::{precompile::B160, primitives::ruint::{Uint, Bits, self}};
use reth_rpc_types::{Filter, ValueOrArray, FilterBlockOption, Topic};


use reth_revm::{primitives::ruint::{aliases::B256}};
use reth_primitives::serde_helper::JsonStorageKey;


pub fn ethers_block_id_to_reth_block_id(block_id: EthersBlockId) -> BlockId {
    match block_id {
        EthersBlockId::Hash(hash) => BlockId::Hash(BlockHash::from_slice(hash.as_bytes()).into()),
        EthersBlockId::Number(number) => BlockId::Number(BlockNumberOrTag::Number(
            number.as_number().unwrap().as_u64(),
        )),
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
    opt_access_list: Option<Vec<reth_primitives::AccessListItem>>,
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




pub fn ethers_typed_transaction_to_reth_call_request(tx: &TypedTransaction) -> CallRequest {
    CallRequest {
        from: tx.from().map(|addr| Address::from_slice(addr.as_bytes())),
        to: tx.to().map(|addr| {
            Address::from_slice(match addr {
                NameOrAddress::Address(addr) => addr.as_bytes(),
                NameOrAddress::Name(_) => panic!(),
            })
        }),
        gas_price: tx.gas_price().map(|v| U256::from_limbs(v.0)),
        max_fee_per_gas: tx.gas_price().map(|v| U256::from_limbs(v.0)),
        max_priority_fee_per_gas: None,
        gas: tx.gas().map(|v| U256::from_limbs(v.0)),
        value: tx.value().map(|v| U256::from_limbs(v.0)),
        data: tx.data().map(|data| Bytes(data.0.clone())),
        nonce: tx.nonce().map(|v| U256::from_limbs(v.0)),
        chain_id: tx.chain_id(),
        access_list: tx
            .access_list()
            .map(|list| ethers_access_list_to_reth_access_list(list.clone())),
        transaction_type: match tx {
            TypedTransaction::Legacy(_) => None,
            TypedTransaction::Eip2930(_) => Some(U8::from(0x1)),
            TypedTransaction::Eip1559(_) => Some(U8::from(0x2)),
        },
    }
}



pub fn reth_rpc_transaction_to_ethers(reth_tx: reth_rpc_types::Transaction) -> EthersTransaction {
    let v = reth_tx.signature.map_or(0.into(), |sig| sig.v.into());
    let r = reth_tx.signature.map_or(0.into(), |sig| sig.r.into());
    let s = reth_tx.signature.map_or(0.into(), |sig| sig.s.into());

    EthersTransaction {
        hash: reth_tx.hash.into(),
        nonce: reth_tx.nonce.into(),
        block_hash: reth_tx.block_hash.into(),
        block_number: reth_tx.block_number.map(|n| n.low_u64().into()),
        transaction_index: reth_tx.transaction_index.map(|n| n.low_u64().into()),
        from: reth_tx.from.into(),
        to: reth_tx.to,
        value: reth_tx.value.into(),
        gas_price: reth_tx.gas_price.map(|p| p.into()),
        gas: reth_tx.gas.into(),
        input: reth_tx.input,
        v: v,
        r: r,
        s: s,
        transaction_type: reth_tx.transaction_type,
        access_list: Some(opt_reth_access_list_to_ethers_access_list(reth_tx.access_list)),
        max_priority_fee_per_gas: reth_tx.max_priority_fee_per_gas.map(|p| p.into()),
        max_fee_per_gas: reth_tx.max_fee_per_gas.map(|p| p.into()),
        chain_id: reth_tx.chain_id.map(|id| id.into()),
        ..Default::default()
    }
}


fn convert_block_number_to_block_number_or_tag(block: EthersBlockNumber) -> Result<BlockNumberOrTag> {
    match block {
        ethers::types::BlockNumber::Latest => Ok(BlockNumberOrTag::Latest),
        ethers::types::BlockNumber::Finalized => Ok(BlockNumberOrTag::Finalized),
        ethers::types::BlockNumber::Safe => Ok(BlockNumberOrTag::Safe),
        ethers::types::BlockNumber::Earliest => Ok(BlockNumberOrTag::Earliest),
        ethers::types::BlockNumber::Pending => Ok(BlockNumberOrTag::Pending),
        ethers::types::BlockNumber::Number(n) => Ok(BlockNumberOrTag::Number(n.as_u64())),
    }
}


fn convert_topics(topics: [Option<EthersTopic>; 4]) -> [Option<Topic>; 4] {
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
    U: From<T>
 {
    match val {
        EthersValueOrArray::Value(Some(addr)) => ValueOrArray::Value(Some(addr.clone().into())),
        EthersValueOrArray::Value(None) => ValueOrArray::Value(None),
        EthersValueOrArray::Array(addrs) => ValueOrArray::Array(addrs.into_iter().map(|a| a.map(U::from)).collect())
    }
}



fn convert_valueORarray<T, U>(val: &EthersValueOrArray<T>) -> ValueOrArray<U>
where
    T: Clone,
    U: From<T>
 {
    match val {
        EthersValueOrArray::Value(addr) => ValueOrArray::Value(addr.clone().into()),
        EthersValueOrArray::Array(addrs) => ValueOrArray::Array(addrs.into_iter().map(|a| Into::<U>::into(a.clone())).collect())
    }
}

/// ---------------------------



pub fn ethers_filter_to_reth_filter(filter: &EthersFilter) -> Filter {

    return Filter {
        block_option: match filter.block_option {
            EthersFilterBlockOption::AtBlockHash(x) => FilterBlockOption::AtBlockHash(x.into()),
            EthersFilterBlockOption::Range{from_block, to_block} => FilterBlockOption::Range { 
                    from_block: convert_block_number_to_block_number_or_tag(from_block.unwrap()).ok(), 
                    to_block: convert_block_number_to_block_number_or_tag(to_block.unwrap()).ok(), 
            },
        },

        address: match &filter.address { 
            Some(addr) => Some(convert_valueORarray(addr)),
            None => None
            },

        topics: convert_topics(filter.topics)


    }
}




pub fn reth_transaction_receipt_to_ethers(
    receipt: TransactionReceipt,
) -> EthersTransactionReceipt {
    EthersTransactionReceipt {
        transaction_hash: receipt.transaction_hash.into().unwrap(),
        transaction_index: receipt.transaction_index.unwrap().as_u64().into(),
        block_hash: receipt.block_hash,
        block_number: receipt.block_number.map(|num| num.as_u64().into()),
        from: receipt.from,
        to: receipt.to,
        cumulative_gas_used: receipt.cumulative_gas_used,
        gas_used: receipt.gas_used,
        contract_address: receipt.contract_address,
        logs: receipt.logs,
        status: receipt.status_code.map(|num| num.as_u64().into()),
        root: receipt.state_root,
        logs_bloom: receipt.logs_bloom,
        transaction_type: Some(receipt.transaction_type.into()),
        effective_gas_price: Some(
            U256::from(receipt.effective_gas_price),
        ),
        other: OtherFields::default(),
    }
}



pub fn convert_location_to_json_key(location: EthersH256) -> JsonStorageKey {
    let location = location.to_fixed_bytes(); 
    let location_u256: U256 = U256::from_be_bytes(location);
    JsonStorageKey::from(location_u256)
}

