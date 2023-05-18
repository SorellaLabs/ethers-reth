use super::{ToEthers, ToReth};

use ethers::types::{BlockId as EthersBlockId, BlockNumber as EthersBlockNumber};
use reth_primitives::{BlockId, BlockNumberOrTag};

/// BlockId (ethers) -> (reth)
impl ToReth<BlockId> for EthersBlockId {
    fn into_reth(self) -> BlockId {
        match self {
            EthersBlockId::Hash(hash) => BlockId::Hash(hash.into_reth().into()),
            EthersBlockId::Number(number) => {
                BlockId::Number(BlockNumberOrTag::Number(number.as_number().unwrap().as_u64()))
            }
        }
    }
}

/// BlockId (reth) -> (ethers)
impl ToEthers<EthersBlockId> for BlockId {
    fn into_ethers(self) -> EthersBlockId {
        match self {
            BlockId::Hash(hash) => EthersBlockId::Hash(hash.block_hash.into_ethers()),
            BlockId::Number(number) => EthersBlockId::Number(number.into_ethers()),
        }
    }
}

// -----------------------------------------------

/// BlockNumber (ethers) -> (reth)
impl ToReth<BlockNumberOrTag> for EthersBlockNumber {
    fn into_reth(self) -> BlockNumberOrTag {
        match self {
            EthersBlockNumber::Latest => BlockNumberOrTag::Latest,
            EthersBlockNumber::Finalized => BlockNumberOrTag::Finalized,
            EthersBlockNumber::Safe => BlockNumberOrTag::Safe,
            EthersBlockNumber::Earliest => BlockNumberOrTag::Earliest,
            EthersBlockNumber::Pending => BlockNumberOrTag::Pending,
            EthersBlockNumber::Number(n) => BlockNumberOrTag::Number(n.as_u64()),
        }
    }
}

/// BlockNumber (reth) -> (ethers)
impl ToEthers<EthersBlockNumber> for BlockNumberOrTag {
    fn into_ethers(self) -> EthersBlockNumber {
        match self {
            BlockNumberOrTag::Latest => EthersBlockNumber::Latest,
            BlockNumberOrTag::Earliest => EthersBlockNumber::Earliest,
            BlockNumberOrTag::Pending => EthersBlockNumber::Pending,
            BlockNumberOrTag::Finalized => EthersBlockNumber::Finalized,
            BlockNumberOrTag::Safe => EthersBlockNumber::Safe,
            BlockNumberOrTag::Number(n) => EthersBlockNumber::Number(n.into()),
        }
    }
}
