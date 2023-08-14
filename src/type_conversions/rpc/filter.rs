use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    Filter as EthersFilter, FilterBlockOption as EthersFilterBlockOption,
    ValueOrArray as EthersValueOrArray, H256 as EthersH256,
};
use reth_primitives::H256;
use reth_rpc_types::{Filter, FilterBlockOption, ValueOrArray};

/// BlockNumber (ethers) -> BlockNumberOrTag (reth)
impl ToReth<FilterBlockOption> for EthersFilterBlockOption {
    fn into_reth(self) -> FilterBlockOption {
        match self {
            EthersFilterBlockOption::Range { from_block, to_block } => FilterBlockOption::Range {
                from_block: from_block.into_reth(),
                to_block: to_block.into_reth(),
            },
            EthersFilterBlockOption::AtBlockHash(hash) => {
                FilterBlockOption::AtBlockHash(hash.into_reth())
            }
        }
    }
}

/// BlockNumberOrTag (reth) -> BlockNumber (ethers)
impl ToEthers<EthersFilterBlockOption> for FilterBlockOption {
    fn into_ethers(self) -> EthersFilterBlockOption {
        match self {
            FilterBlockOption::Range { from_block, to_block } => EthersFilterBlockOption::Range {
                from_block: from_block.into_ethers(),
                to_block: to_block.into_ethers(),
            },
            FilterBlockOption::AtBlockHash(hash) => {
                EthersFilterBlockOption::AtBlockHash(hash.into_ethers())
            }
        }
    }
}

// -----------------------------------------------

/// ValueOrArray (ethers) -> (reth)
impl<T, U> ToReth<ValueOrArray<T>> for EthersValueOrArray<U>
where
    U: ToReth<T>,
{
    fn into_reth(self) -> ValueOrArray<T> {
        match self {
            EthersValueOrArray::Value(addr) => ValueOrArray::Value(addr.into_reth()),
            EthersValueOrArray::Array(addrs) => ValueOrArray::Array(addrs.into_reth()),
        }
    }
}

/// ValueOrArray (reth) -> (ethers)
impl<U, T> ToEthers<EthersValueOrArray<U>> for ValueOrArray<T>
where
    T: ToEthers<U>,
{
    fn into_ethers(self) -> EthersValueOrArray<U> {
        match self {
            ValueOrArray::Value(addr) => EthersValueOrArray::Value(addr.into_ethers()),
            ValueOrArray::Array(addrs) => EthersValueOrArray::Array(addrs.into_ethers()),
        }
    }
}

// -----------------------------------------------

/// Filter (ethers) + (reth)
impl ToReth<Filter> for EthersFilter {
    fn into_reth(self) -> Filter {
        Filter {
            block_option: self.block_option.into_reth(),
            address: self.address.map(|a| a.into_reth().into()).unwrap_or_default(),
            topics: self.topics.map(|t| t.map(|a| a.into_reth().into()).unwrap_or_default()),
        }
    }
}

// Helper for converting H256 (reth) to Option<EthersH256> (ethers)
impl ToEthers<Option<EthersH256>> for H256 {
    fn into_ethers(self) -> Option<EthersH256> {
        Some(self.into_ethers())
    }
}

/// Filter (reth) -> (ethers)
impl ToEthers<EthersFilter> for Filter {
    fn into_ethers(self) -> EthersFilter {
        EthersFilter {
            block_option: self.block_option.into_ethers(),
            address: self.address.to_value_or_array().into_ethers(),
            topics: self.topics.map(|t| t.to_value_or_array().into_ethers()),
        }
    }
}
