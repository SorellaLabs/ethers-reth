use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    Block as EthersBlock, OtherFields, Transaction as EthersTransaction, H256 as EthersH256,
};
use reth_rpc_types::{Block, BlockTransactions, Header, Rich};

/// EthersBlock<EthersH256> (ethers) -> Rich<Block> (reth)
impl ToReth<Rich<Block>> for EthersBlock<EthersH256> {
    fn into_reth(self) -> Rich<Block> {
        let txs = self.transactions;
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
                blob_gas_used: None,            // @todo
                excess_blob_gas: None,          // @todo
                parent_beacon_block_root: None, // @todo
            },
            total_difficulty: self.total_difficulty.into_reth(),
            uncles: self.uncles.into_reth(),
            transactions: txs.into_reth(),
            size: self.size.into_reth(),
            withdrawals: self.withdrawals.into_reth(),
        };
        Rich::from(block)
    }
}

/// EthersBlock<EthersTransaction> (ethers) -> Rich<Block> (reth)
impl ToReth<Rich<Block>> for EthersBlock<EthersTransaction> {
    fn into_reth(self) -> Rich<Block> {
        let txs = self.transactions;
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
                blob_gas_used: None,            // @todo
                excess_blob_gas: None,          // @todo
                parent_beacon_block_root: None, // @todo
            },
            total_difficulty: self.total_difficulty.into_reth(),
            uncles: self.uncles.into_reth(),
            transactions: txs.into_reth(),
            size: self.size.into_reth(),
            withdrawals: self.withdrawals.into_reth(),
        };
        Rich::from(block)
    }
}

// ---------------------------------------

/// Rich<Block> (reth) -> EthersBlock<EthersTransaction> (ethers)
impl ToEthers<EthersBlock<EthersH256>> for Rich<Block> {
    fn into_ethers(self) -> EthersBlock<EthersH256> {
        let txs = match &self.transactions {
            BlockTransactions::Hashes(hashes) => hashes.into_ethers(),
            _ => vec![],
        };
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
            extra_data: self.header.extra_data.clone().into_ethers(),
            logs_bloom: Some(self.header.logs_bloom.into_ethers()),
            timestamp: self.header.timestamp.into_ethers(),
            difficulty: self.header.difficulty.into_ethers(),
            total_difficulty: self.inner.total_difficulty.into_ethers(),
            seal_fields: vec![],
            uncles: self.inner.uncles.clone().into_ethers(),
            transactions: txs,
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

/// Rich<Block> (reth) -> EthersBlock<EthersTransaction> (ethers)
impl ToEthers<EthersBlock<EthersTransaction>> for Rich<Block> {
    fn into_ethers(self) -> EthersBlock<EthersTransaction> {
        let txs = match &self.transactions {
            BlockTransactions::Full(txs) => txs.into_ethers(),
            _ => vec![],
        };
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
            extra_data: self.header.extra_data.clone().into_ethers(),
            logs_bloom: Some(self.header.logs_bloom.into_ethers()),
            timestamp: self.header.timestamp.into_ethers(),
            difficulty: self.header.difficulty.into_ethers(),
            total_difficulty: self.inner.total_difficulty.into_ethers(),
            seal_fields: vec![],
            uncles: self.inner.uncles.clone().into_ethers(),
            transactions: txs,
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

/// EthersBlock<H256> (ethers) -> BlockTransactions (reth)
impl ToReth<BlockTransactions> for Vec<EthersH256> {
    fn into_reth(self) -> BlockTransactions {
        BlockTransactions::Hashes(self.into_reth())
    }
}

/// EthersBlock<Transaction> (ethers) -> BlockTransactions (reth)
impl ToReth<BlockTransactions> for Vec<EthersTransaction> {
    fn into_reth(self) -> BlockTransactions {
        BlockTransactions::Full(self.into_reth())
    }
}
