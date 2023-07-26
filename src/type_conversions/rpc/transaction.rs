use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    OtherFields, Transaction as EthersTransaction, TransactionReceipt as EthersTransactionReceipt,
};
use reth_primitives::{AccessList, Signature as PrimitiveSignature, TxType as PrimitiveTxType};
use reth_revm::primitives::ruint::Uint;
use reth_rpc_types::{Signature, Transaction, TransactionReceipt};

/// Transaction (ethers) -> (reth)
impl ToReth<Transaction> for EthersTransaction {
    fn into_reth(self) -> Transaction {
        let primitive_tx_type = match self.transaction_type.map(|t| t.as_u64()) {
            None => PrimitiveTxType::Legacy,
            Some(1) => PrimitiveTxType::EIP2930,
            Some(2) => PrimitiveTxType::EIP1559,
            _ => {
                unimplemented!("Missing tx type");
            }
        };

        let v = self.v.as_u64();
        let normalized_v = if v > 1 {
            v - self.chain_id.expect("Should not have unnormalized v without chain id").as_u64() * 2
                - 35
        } else {
            v
        };

        assert!((v == 0) | (v == 1));

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
            signature: Some(Signature::from_primitive_signature(
                primitive_signature,
                primitive_tx_type,
                self.chain_id.map(|t| t.as_u64()),
            )),
            chain_id: self.chain_id.into_reth(),
            access_list: self.access_list.map(|a| a.into_reth().0),
            transaction_type: self.transaction_type,
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
            transaction_index: Some(self.transaction_index.into_reth()),
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
        }
    }
}

/// TransactionReceipt (reth) -> (ethers)
impl ToEthers<EthersTransactionReceipt> for TransactionReceipt {
    fn into_ethers(self) -> EthersTransactionReceipt {
        EthersTransactionReceipt {
            transaction_hash: self.transaction_hash.into_ethers().unwrap(),
            transaction_index: self.transaction_index.into_ethers().unwrap(),
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
