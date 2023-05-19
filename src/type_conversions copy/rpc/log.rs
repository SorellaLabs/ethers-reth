use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::Log as EthersLog;
use reth_rpc_types::Log;

/// Log (ethers) -> (reth)
impl ToReth<Log> for EthersLog {
    fn into_reth(self) -> Log {
        Log {
            address: self.address.into_reth(),
            topics: self.topics.into_reth(),
            data: self.data.into_reth(),
            block_hash: self.block_hash.into_reth(),
            block_number: self.block_number.into_reth(),
            transaction_hash: self.transaction_hash.into_reth(),
            transaction_index: self.transaction_index.into_reth(),
            log_index: self.log_index.into_reth(),
            removed: self.removed.is_some(),
        }
    }
}

/// Log (reth) -> (ethers)
impl ToEthers<EthersLog> for Log {
    fn into_ethers(self) -> EthersLog {
        EthersLog {
            address: self.address.into_ethers(),
            topics: self.topics.into_ethers(),
            data: self.data.into_ethers(),
            block_hash: self.block_hash.into_ethers(),
            block_number: self.block_number.into_ethers(),
            transaction_hash: self.transaction_hash.into_ethers(),
            transaction_index: self.transaction_index.into_ethers(),
            log_index: self.log_index.into_ethers(),
            transaction_log_index: None,
            log_type: None,
            removed: Some(self.removed),
        }
    }
}
