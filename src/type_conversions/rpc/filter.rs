use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    Filter as EthersFilter, FilterBlockOption as EthersFilterBlockOption,
    ValueOrArray as EthersValueOrArray,
};
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
