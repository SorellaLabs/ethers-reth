use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    OtherFields, Transaction as EthersTransaction, TransactionReceipt as EthersTransactionReceipt,
};
use reth_primitives::{AccessList, Signature as PrimitiveSignature, U256, U64};
use reth_revm::primitives::ruint::Uint;
use reth_rpc_types::{Parity, Signature, Transaction, TransactionReceipt};

/// Transaction (ethers) -> (reth)
impl ToReth<Transaction> for EthersTransaction {
    fn into_reth(self) -> Transaction {
        let v = self.v.as_u64();
        let normalized_v = if v > 1 {
            v - self.chain_id.expect("Should not have unnormalized v without chain id").as_u64() * 2 -
                35
        } else {
            v
        };

        assert!((normalized_v == 0) | (normalized_v == 1));

        let primitive_signature = PrimitiveSignature {
            r: self.r.into_reth(),
            s: self.s.into_reth(),
            odd_y_parity: normalized_v == 1,
        };

        Transaction {
            hash: self.hash.into_reth(),
            nonce: self.nonce.into_reth(),
            block_hash: self.block_hash.into_reth(),
            block_number: self.block_number.into_reth(),
            transaction_index: self.transaction_index.into_reth(),
            from: self.from.into_reth(),
            to: self.to.into_reth(),
            value: self.value.into_reth(),
            gas_price: self.gas_price.into_reth(),
            gas: self.gas.into_reth(),
            max_fee_per_gas: self.max_fee_per_gas.into_reth(),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas.into_reth(),
            input: self.input.into_reth(),
            signature: Some(Signature {
                r: primitive_signature.r,
                s: primitive_signature.s,
                v: U256::from(
                    primitive_signature.v(self.chain_id.into_reth().map(|v: U64| v.as_u64())),
                ),
                y_parity: Some(Parity(primitive_signature.odd_y_parity)),
            }),
            chain_id: self.chain_id.into_reth(),
            access_list: self.access_list.map(|a| a.into_reth().0),
            transaction_type: self.transaction_type,
            max_fee_per_blob_gas: None,
            blob_versioned_hashes: vec![],
        }
    }
}

/// Transaction (reth) -> (ethers)
impl ToEthers<EthersTransaction> for Transaction {
    fn into_ethers(self) -> EthersTransaction {
        let (v, r, s) =
            self.signature.map_or((Uint::MIN, Uint::MIN, Uint::MIN), |sig| (sig.v, sig.r, sig.s));

        EthersTransaction {
            hash: self.hash.into_ethers(),
            nonce: self.nonce.into_ethers(),
            block_hash: self.block_hash.into_ethers(),
            block_number: self.block_number.into_ethers(),
            transaction_index: self.transaction_index.into_ethers(),
            from: self.from.into_ethers(),
            to: self.to.into_ethers(),
            value: self.value.into_ethers(),
            gas_price: self.gas_price.into_ethers(),
            gas: self.gas.into_ethers(),
            input: self.input.into_ethers(),
            v: v.into_ethers(),
            r: r.into_ethers(),
            s: s.into_ethers(),
            transaction_type: self.transaction_type,
            access_list: self.access_list.map(|acc| AccessList(acc).into_ethers()),
            max_priority_fee_per_gas: self.max_priority_fee_per_gas.into_ethers(),
            max_fee_per_gas: self.max_fee_per_gas.into_ethers(),
            chain_id: self.chain_id.into_ethers(),
            ..Default::default()
        }
    }
}

// -----------------------------------------------

/// TransactionReceipt (ethers) -> (reth)
impl ToReth<TransactionReceipt> for EthersTransactionReceipt {
    fn into_reth(self) -> TransactionReceipt {
        TransactionReceipt {
            transaction_hash: Some(self.transaction_hash.into_reth()),
            transaction_index: self.transaction_index.into_reth(),
            block_hash: self.block_hash.into_reth(),
            block_number: self.block_number.into_reth(),
            from: self.from.into_reth(),
            to: self.to.into_reth(),
            cumulative_gas_used: self.cumulative_gas_used.into_reth(),
            gas_used: self.gas_used.into_reth(),
            contract_address: self.contract_address.into_reth(),
            logs: self.logs.into_reth(),
            status_code: self.status.into_reth(),
            state_root: self.root.into_reth(),
            logs_bloom: self.logs_bloom.into_reth(),
            transaction_type: self.transaction_type.into_reth().unwrap(),
            effective_gas_price: self.effective_gas_price.into_reth().unwrap(),
            blob_gas_used: None,
            blob_gas_price: None,
        }
    }
}

/// TransactionReceipt (reth) -> (ethers)
impl ToEthers<EthersTransactionReceipt> for TransactionReceipt {
    fn into_ethers(self) -> EthersTransactionReceipt {
        EthersTransactionReceipt {
            transaction_hash: self.transaction_hash.into_ethers().unwrap(),
            transaction_index: self.transaction_index.into_ethers(),
            block_hash: self.block_hash.into_ethers(),
            block_number: self.block_number.into_ethers(),
            from: self.from.into_ethers(),
            to: self.to.into_ethers(),
            cumulative_gas_used: self.cumulative_gas_used.into_ethers(),
            gas_used: self.gas_used.into_ethers(),
            contract_address: self.contract_address.into_ethers(),
            logs: self.logs.into_ethers(),
            status: self.status_code.into_ethers(),
            root: self.state_root.into_ethers(),
            logs_bloom: self.logs_bloom.into_ethers(),
            transaction_type: Some(self.transaction_type.into_ethers()),
            effective_gas_price: Some(self.effective_gas_price.into_ethers()),
            other: OtherFields::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::type_conversions::{ToEthers, ToReth};

    use ethers::types::{
        Bytes as EthersBytes, Transaction as EthersTransaction,
        TransactionReceipt as EthersTransactionReceipt, H160 as EthersH160, H256 as EthersH256,
        U256 as EthersU256, U64 as EthersU64,
    };

    use reth_primitives::{hex_literal::hex, Address, Bloom, Bytes, H256, U128, U256, U64, U8};
    use reth_rpc_types::{Parity, Signature, Transaction, TransactionReceipt};

    #[test]
    fn transaction() {
        let r: Transaction = Transaction {
            hash: H256::from_low_u64_be(1),
            nonce: U256::from(2),
            block_hash: Some(H256::from_low_u64_be(3)),
            block_number: Some(U256::from(4)),
            transaction_index: Some(U256::from(5)),
            from: Address::from_low_u64_be(6),
            to: Some(Address::from_low_u64_be(7)),
            value: U256::from(8),
            gas_price: Some(U128::from(9)),
            gas: U256::from(10),
            input: Bytes::from(vec![11, 12, 13]),
            signature: Some(Signature {
                r: U256::from(14),
                s: U256::from(14),
                v: U256::from(38),
                y_parity: Some(Parity(true)),
            }),
            chain_id: Some(U64::from(1)),
            access_list: None,
            transaction_type: Some(U64::from(2)),
            max_fee_per_gas: Some(U128::from(21)),
            max_priority_fee_per_gas: Some(U128::from(22)),
            max_fee_per_blob_gas: None,
            blob_versioned_hashes: vec![],
        };
        let e: EthersTransaction = EthersTransaction {
            hash: EthersH256::from_str(
                "0x0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap(),
            nonce: EthersU256::from(2),
            block_hash: Some(
                EthersH256::from_str(
                    "0x0000000000000000000000000000000000000000000000000000000000000003",
                )
                .unwrap(),
            ),
            block_number: Some(EthersU64::from(4)),
            transaction_index: Some(EthersU64::from(5)),
            from: EthersH160::from_str("0x0000000000000000000000000000000000000006").unwrap(),
            to: Some(EthersH160::from_str("0x0000000000000000000000000000000000000007").unwrap()),
            value: EthersU256::from(8),
            gas_price: Some(EthersU256::from(9)),
            gas: EthersU256::from(10),
            input: EthersBytes::from(vec![11, 12, 13]),
            v: EthersU64::from(38),
            r: EthersU256::from(14),
            s: EthersU256::from(14),
            chain_id: Some(EthersU256::from(1)),
            access_list: None,
            transaction_type: Some(EthersU64::from(2)),
            max_fee_per_gas: Some(EthersU256::from(21)),
            max_priority_fee_per_gas: Some(EthersU256::from(22)),
            other: Default::default(),
        };

        assert_eq!(r, e.clone().into_reth());
        assert_eq!(e, r.into_ethers());
    }

    #[test]
    fn transaction_receipt() {
        let bloom_hex = hex!(
            "000000000000000000810000000000000000000000000000000000020000000000000000000000000000008000"
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
            "000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000"
            "000000000000000000000000000000000000000000000000000000280000000000400000800000004000000000"
            "000000000000000000000000000000000000000000000000000000000000100000100000000000000000000000"
            "00000000001400000000000000008000000000000000000000000000000000"
        );
        let r: TransactionReceipt = TransactionReceipt {
            transaction_hash: Some(H256::from_low_u64_be(1)),
            transaction_index: U64::from(2),
            block_hash: Some(H256::from_low_u64_be(3)),
            block_number: Some(U256::from(4)),
            from: Address::from_low_u64_be(6),
            to: Some(Address::from_low_u64_be(7)),
            cumulative_gas_used: U256::from(8),
            gas_used: Some(U256::from(9)),
            contract_address: Some(Address::from_low_u64_be(10)),
            logs: vec![],
            state_root: Some(H256::from_low_u64_be(11)),
            logs_bloom: Bloom::from_slice(&bloom_hex),
            status_code: Some(U64::from(15)),
            effective_gas_price: U128::from(16),
            transaction_type: U8::from(0),
            blob_gas_used: None,
            blob_gas_price: None,
        };
        let e: EthersTransactionReceipt = EthersTransactionReceipt {
            transaction_hash: H256::from_low_u64_be(1).into_ethers(),
            transaction_index: EthersU64::from(2),
            block_hash: Some(H256::from_low_u64_be(3).into_ethers()),
            block_number: Some(EthersU64::from(4)),
            from: Address::from_low_u64_be(6).into_ethers(),
            to: Some(Address::from_low_u64_be(7).into_ethers()),
            cumulative_gas_used: EthersU256::from(8),
            gas_used: Some(EthersU256::from(9)),
            contract_address: Some(Address::from_low_u64_be(10).into_ethers()),
            logs: vec![],
            logs_bloom: Bloom::from_slice(&bloom_hex).into_ethers(),
            status: Some(EthersU64::from(15)),
            root: Some(H256::from_low_u64_be(11).into_ethers()),
            transaction_type: Some(EthersU64::from(0)),
            effective_gas_price: Some(EthersU256::from(16)),
            other: Default::default(),
        };

        assert_eq!(r, e.clone().into_reth());
        assert_eq!(e, r.into_ethers());
    }
}
