use super::{ToEthers, ToReth};

use ethers::types::Withdrawal as EthersWithdrawal;
use reth_primitives::Withdrawal;

/// Withdrawal (ethers) + (reth)
impl ToReth<Withdrawal> for EthersWithdrawal {
    fn into_reth(self) -> Withdrawal {
        Withdrawal {
            index: self.index.as_u64(),
            validator_index: self.index.as_u64(),
            address: self.address.into_reth(),
            amount: self.amount.as_u64(),
        }
    }
}

impl ToEthers<EthersWithdrawal> for Withdrawal {
    fn into_ethers(self) -> EthersWithdrawal {
        EthersWithdrawal {
            index: self.index.into(),
            validator_index: self.index.into(),
            address: self.address.into(),
            amount: self.amount.into(),
        }
    }
}
