use ethers::types::Withdrawal as EthersWithdrawal;
use reth_primitives::Withdrawal;

use super::{ToEthers, ToReth};

/// Withdrawal (ethers) -> (reth)
impl ToReth<Withdrawal> for EthersWithdrawal {
    fn into_reth(self) -> Withdrawal {
        Withdrawal {
            index: self.index.as_u64(),
            validator_index: self.validator_index.as_u64(),
            address: self.address.into_reth(),
            amount: self.amount.as_u64(),
        }
    }
}

/// Withdrawal (reth) -> (ethers)
impl ToEthers<EthersWithdrawal> for Withdrawal {
    fn into_ethers(self) -> EthersWithdrawal {
        EthersWithdrawal {
            index: self.index.into(),
            validator_index: self.validator_index.into(),
            address: self.address.into(),
            amount: self.amount.into(),
        }
    }
}

mod tests {
    use std::str::FromStr;

    use ethers::prelude::Address;
    use ethers::types::Address as EthersAddress;
    // use ethers::types::Address as EthersAddress;
    use reth_primitives::H160;

    use super::*;

    const WETH_ADDRESS_MAINNET: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    const ONE_ETH_IN_WEI: u64 = 1000000000000000000;
    const WITHDRAWAL_INDEX: u64 = 10;
    const VALIDATOR_INDEX: u64 = 20;

    #[test]
    fn test_ethers_withdrawal_to_reth_withdrawal_conversion() {
        let ethers_withdrawal = EthersWithdrawal {
            index: WITHDRAWAL_INDEX.into(),
            validator_index: VALIDATOR_INDEX.into(),
            address: EthersAddress::from_str(WETH_ADDRESS_MAINNET).unwrap(),
            amount: ONE_ETH_IN_WEI.into(),
        };

        let reth_withdrawal: Withdrawal = ethers_withdrawal.into_reth();

        assert_eq!(reth_withdrawal.index, WITHDRAWAL_INDEX);
        assert_eq!(reth_withdrawal.validator_index, VALIDATOR_INDEX);
        assert_eq!(reth_withdrawal.address, H160::from_str(WETH_ADDRESS_MAINNET).unwrap());
        assert_eq!(reth_withdrawal.amount, ONE_ETH_IN_WEI);
    }

    #[test]
    fn test_reth_withdrawal_to_ethers_withdrawal_conversion() {
        let reth_withdrawal = Withdrawal {
            index: WITHDRAWAL_INDEX,
            validator_index: VALIDATOR_INDEX,
            address: H160::from_str(WETH_ADDRESS_MAINNET).unwrap(),
            amount: ONE_ETH_IN_WEI,
        };

        let ethers_withdrawal: EthersWithdrawal = reth_withdrawal.into_ethers();

        assert_eq!(ethers_withdrawal.index, WITHDRAWAL_INDEX.into());
        assert_eq!(ethers_withdrawal.validator_index, VALIDATOR_INDEX.into());
        assert_eq!(ethers_withdrawal.address, Address::from_str(WETH_ADDRESS_MAINNET).unwrap());
        assert_eq!(ethers_withdrawal.amount, ONE_ETH_IN_WEI.into());
    }
}
