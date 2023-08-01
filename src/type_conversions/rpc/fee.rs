use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::FeeHistory as EthersFeeHistory;
use reth_rpc_types::FeeHistory;

// FeeHistory (ethers) -> (reth)
impl ToReth<FeeHistory> for EthersFeeHistory {
    fn into_reth(self) -> FeeHistory {
        FeeHistory {
            base_fee_per_gas: self.base_fee_per_gas.into_reth(),
            gas_used_ratio: self.gas_used_ratio,
            oldest_block: self.oldest_block.into_reth(),
            reward: Some(self.reward.into_reth()),
        }
    }
}

/// FeeHistory (reth) -> (ethers)
impl ToEthers<EthersFeeHistory> for FeeHistory {
    fn into_ethers(self) -> EthersFeeHistory {
        EthersFeeHistory {
            base_fee_per_gas: self.base_fee_per_gas.into_ethers(),
            gas_used_ratio: self.gas_used_ratio,
            oldest_block: self.oldest_block.into_ethers(),
            reward: self.reward.into_ethers(),
        }
    }
}
