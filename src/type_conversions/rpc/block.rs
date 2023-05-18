use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{Block as EthersBlock, OtherFields};
use reth_primitives::H256;
use reth_rpc_types::{Block, BlockTransactions, Header, Rich, RichBlock, Transaction};

/// FeeHistory (ethers) + (reth)
impl<TX> ToReth<RichBlock> for EthersBlock<TX> {
    fn into_reth(self) -> RichBlock {
        let block = Block {
            header: Header {
                hash: self.hash.into_reth(),
                parent_hash: self.parent_hash.into_reth(),
                uncles_hash: self.uncles_hash.into_reth(),
                miner: self.author.into_reth().unwrap(),
                state_root: self.state_root.into_reth(),
                transactions_root: self.transactions_root.into_reth(),
                receipts_root: self.receipts_root.into_reth(),
                logs_bloom: self.logs_bloom.into_reth().unwrap(),
                difficulty: self.difficulty.into_reth(),
                number: self.number.into_reth(),
                gas_limit: self.gas_limit.into_reth(),
                gas_used: self.gas_used.into_reth(),
                timestamp: self.timestamp.into_reth(),
                extra_data: self.extra_data.into_reth(),
                mix_hash: self.mix_hash.into_reth().unwrap(),
                nonce: self.nonce.into_reth(),
                base_fee_per_gas: self.base_fee_per_gas.into_reth(),
                withdrawals_root: self.withdrawals_root.into_reth(),
            },
            total_difficulty: self.total_difficulty.into_reth(),
            uncles: self.uncles.into_reth(),
            transactions: self.transactions.into_reth(),
            size: self.size.into_reth(),
            withdrawals: self.withdrawals.into_reth(),
        };
        Rich::from(block)
    }
}

impl<TX> ToEthers<EthersBlock<TX>> for RichBlock
where
    BlockTransactions: ToEthers<Vec<TX>>,
{
    fn into_ethers(self) -> EthersBlock<TX> {
        EthersBlock {
            hash: self.header.hash.into_ethers(),
            parent_hash: self.header.parent_hash.into_ethers(),
            uncles_hash: self.header.uncles_hash.into_ethers(),
            author: Some(self.header.miner.into_ethers()),
            state_root: self.header.state_root.into_ethers(),
            transactions_root: self.header.transactions_root.into_ethers(),
            receipts_root: self.header.receipts_root.into_ethers(),
            number: self.header.number.into_ethers(),
            gas_used: self.header.gas_used.into_ethers(),
            gas_limit: self.header.gas_limit.into_ethers(),
            extra_data: self.header.extra_data.into_ethers(),
            logs_bloom: Some(self.header.logs_bloom.into_ethers()),
            timestamp: self.header.timestamp.into_ethers(),
            difficulty: self.header.difficulty.into_ethers(),
            total_difficulty: self.inner.total_difficulty.into_ethers(),
            seal_fields: vec![],
            uncles: self.inner.uncles.into_ethers(),
            transactions: self.transactions.into_ethers(),
            size: self.inner.size.into_ethers(),
            mix_hash: Some(self.header.mix_hash.into_ethers()),
            nonce: self.header.nonce.into_ethers(),
            base_fee_per_gas: self.header.base_fee_per_gas.into_ethers(),
            withdrawals_root: self.header.withdrawals_root.into_ethers(),
            withdrawals: self.inner.withdrawals.into_ethers(),
            other: OtherFields::default(),
        }
    }
}

// -----------------------------------------------

/// generic BlockTransactions (reth) depending on:
/// TX = Hash, Full
impl<TX> ToReth<BlockTransactions> for Vec<TX> {
    fn into_reth(self) -> BlockTransactions {
        todo!()
    }
}

impl<TX> ToEthers<Vec<TX>> for BlockTransactions
where
    H256: ToEthers<TX>,
    Transaction: ToEthers<TX>,
{
    fn into_ethers(self) -> Vec<TX> {
        match self {
            BlockTransactions::Hashes(hashes) => hashes.into_ethers(),
            BlockTransactions::Full(txs) => txs.into_ethers(),
            BlockTransactions::Uncle => vec![],
        }
    }
}
