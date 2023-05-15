use ethers::types::{
    transaction::{eip2718::TypedTransaction, eip2930::AccessList as EthersAccessList, eip2930::AccessListItem as EthersAccessListItem, eip2930::AccessListWithGasUsed as EthersAccessListWithGasUsed},
    BlockId as EthersBlockId, NameOrAddress,
};
use ethers::types::TransactionReceipt as EthersTransactionReceipt;
use ethers::types::Transaction;
use ethers::types::OtherFields;

use reth_primitives::{
    AccessList, AccessListWithGasUsed, AccessListItem, Address, BlockHash, BlockId, BlockNumberOrTag, Bytes, U256,
    U8, H160,
};
use reth_revm::{precompile::B160, primitives::ruint::{aliases::B256, Uint, Bits}};
use reth_rpc_types::TransactionReceipt;

use reth_rpc_types::{CallRequest, Filter, ValueOrArray, FilterBlockOption, Topic};

pub fn ethers_block_id_to_reth_block_id(block_id: EthersBlockId) -> BlockId {
    match block_id {
        EthersBlockId::Hash(hash) => BlockId::Hash(BlockHash::from_slice(hash.as_bytes()).into()),
        EthersBlockId::Number(number) => BlockId::Number(BlockNumberOrTag::Number(
            number.as_number().unwrap().as_u64(),
        )),
    }
}

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


fn convert_block_number_to_block_number_or_tag(block: ethers::types::BlockNumber) -> BlockNumberOrTag {
    match block {
        ethers::types::BlockNumber::Latest => BlockNumberOrTag::Latest,
        ethers::types::BlockNumber::Finalized => BlockNumberOrTag::Finalized,
        ethers::types::BlockNumber::Safe => BlockNumberOrTag::Safe,
        ethers::types::BlockNumber::Earliest => BlockNumberOrTag::Earliest,
        ethers::types::BlockNumber::Pending => BlockNumberOrTag::Pending,
        ethers::types::BlockNumber::Number(n) => BlockNumberOrTag::Number(n.as_u64()),
    }
}


fn convert_topics(topics: [Option<ethers::types::Topic>; 4]) -> [Option<Topic>; 4] {
    let mut new_topics: [Option<Topic>; 4];
    
    for (i, topic) in topics.into_iter().enumerate() {
        new_topics[i] = match topic {
            Some(t) => Some(convert_valueORarray(t)),
            None => None
        };
    }

    new_topics
}


fn convert_valueORarray<T, U>(val: ethers::types::ValueOrArray<Option<T>>) -> ValueOrArray<Option<U>>
where 
    U: From<T>,
    Vec<U>: FromIterator<T>
{
    match val {
        ethers::types::ValueOrArray::Value(addr) => ValueOrArray::Value( match addr {
            Some(a) => Some(Into::<U>::into(a)),
            None => None
        }),

        ethers::types::ValueOrArray::Array(addrs) => ValueOrArray::Array(addrs.into_iter().map(|addr| { match addr { 
            Some(a) => Some(Into::<U>::into(a)),
            None => None
        }}).collect::<Vec<Option<U>>>())
    }
}


pub fn ethers_filter_to_reth_filter(filter: &ethers::types::Filter) -> Filter {
    Filter {
        block_option: match filter.block_option {
            ethers::types::FilterBlockOption::AtBlockHash(x) => FilterBlockOption::AtBlockHash(x),
            ethers::types::FilterBlockOption::Range { 
                from_block, 
                to_block 
            } => FilterBlockOption::Range { 
                    from_block: match from_block {
                        Some(b) => Some(convert_block_number_to_block_number_or_tag(b)),
                        None => None
                    }, 
                    to_block: match to_block {
                        Some(b) => Some(convert_block_number_to_block_number_or_tag(b)),
                        None => None
                    }, 
                }
        },

        address: match filter.address {
            None => None,
            Some(inner) => match inner {
                ethers::types::ValueOrArray::Value(addr) => Some(ValueOrArray::Value(addr.into())),
                ethers::types::ValueOrArray::Array(addrs) => Some(ValueOrArray::Array(addrs.into_iter().map(|a| a.into()).collect::<Vec<B160>>()))
            }
        },

        topics: match filter.topics {
        }


    }
}



pub fn reth_transaction_to_ethers(tx: reth_rpc_types::Transaction) -> Transaction {
    Transaction {
        hash: H256::from_slice(tx.hash.as_bytes()),
        nonce: U256::from_limbs(tx.nonce.0),
        block_hash: tx.block_hash.map(|hash| H256::from_slice(hash.as_bytes())),
        block_number: tx.block_number.map(|number| U256::from_limbs(number.0)),
        transaction_index: tx.transaction_index.map(|index| U256::from_limbs(index.0)),
        from: Address::from_slice(tx.from.as_bytes()),
        to: tx.to.map(|addr| Address::from_slice(addr.as_bytes())),
        value: U256::from_limbs(tx.value.0),
        gas_price: U256::from_limbs(tx.gas_price.0),
        gas: U256::from_limbs(tx.gas.0),
        input: Bytes(tx.input.0),
        v: U8::from(tx.v),
        r: H256::from_slice(tx.r.as_bytes()),
        s: H256::from_slice(tx.s.as_bytes()),
        access_list: tx
            .access_list
            .map(|list| ethers_access_list_to_reth_access_list(list.clone())),
        transaction_type: match tx.transaction_type {
            Some(0x1) => Some(TxType::Eip2930),
            Some(0x2) => Some(TxType::Eip1559),
            _ => None,
        },
    }
}


pub fn reth_transaction_receipt_to_ethers(
    receipt: TransactionReceipt,
) -> EthersTransactionReceipt {
    EthersTransactionReceipt {
        transaction_hash: receipt.transaction_hash.unwrap(),
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