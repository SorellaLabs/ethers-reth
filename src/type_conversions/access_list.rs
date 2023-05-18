use super::{ToEthers, ToReth};

use ethers::types::transaction::eip2930::{
    AccessList as EthersAccessList, AccessListItem as EthersAccessListItem,
    AccessListWithGasUsed as EthersAccessListWithGasUsed,
};
use reth_primitives::{AccessList, AccessListItem, AccessListWithGasUsed};

/// AccessListItem (ethers) -> (reth)
impl ToReth<AccessListItem> for EthersAccessListItem {
    fn into_reth(self) -> AccessListItem {
        AccessListItem {
            address: self.address.into_reth(),
            storage_keys: self.storage_keys.into_reth(),
        }
    }
}

/// AccessListItem (reth) -> (ethers)
impl ToEthers<EthersAccessListItem> for AccessListItem {
    fn into_ethers(self) -> EthersAccessListItem {
        EthersAccessListItem {
            address: self.address.into_ethers(),
            storage_keys: self.storage_keys.into_ethers(),
        }
    }
}

// -----------------------------------------------

/// AccessList (ethers) -> (reth)
impl ToReth<AccessList> for EthersAccessList {
    fn into_reth(self) -> AccessList {
        AccessList(self.0.into_reth())
    }
}

/// AccessList (reth) -> (ethers)
impl ToEthers<EthersAccessList> for AccessList {
    fn into_ethers(self) -> EthersAccessList {
        EthersAccessList::from(self.0.into_ethers())
    }
}

// -----------------------------------------------

/// AccessListWithGas (ethers) -> (reth)
impl ToReth<AccessListWithGasUsed> for EthersAccessListWithGasUsed {
    fn into_reth(self) -> AccessListWithGasUsed {
        AccessListWithGasUsed {
            access_list: self.access_list.into_reth(),
            gas_used: self.gas_used.into_reth(),
        }
    }
}

/// AccessListWithGas (reth) -> (ethers)
impl ToEthers<EthersAccessListWithGasUsed> for AccessListWithGasUsed {
    fn into_ethers(self) -> EthersAccessListWithGasUsed {
        EthersAccessListWithGasUsed {
            access_list: self.access_list.into_ethers(),
            gas_used: self.gas_used.into_ethers(),
        }
    }
}
