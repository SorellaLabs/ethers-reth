use crate::type_conversions::ToReth;

use ethers::types::{transaction::eip2718::TypedTransaction as EthersTypedTransaction, Bytes};
use reth_revm::primitives::ruint::Uint;
use reth_rpc_types::{CallInput, CallRequest};

/// Bytes (ethers) -> CallInput (reth)
impl ToReth<CallInput> for Bytes {
    fn into_reth(self) -> CallInput {
        CallInput { input: Some(self.into_reth()), data: None }
    }
}

/// Typed Tx (ethers) -> Call Request (reth)
impl ToReth<CallRequest> for EthersTypedTransaction {
    fn into_reth(self) -> CallRequest {
        CallRequest {
            from: self.from().into_reth(),
            to: self.to_addr().into_reth(),
            gas_price: self.gas_price().into_reth(),
            max_fee_per_gas: self
                .as_eip1559_ref()
                .map(|tx| tx.max_fee_per_gas.into_reth())
                .unwrap_or_default(),
            max_priority_fee_per_gas: self
                .as_eip1559_ref()
                .map(|tx| tx.max_priority_fee_per_gas.into_reth())
                .unwrap_or_default(),
            gas: self.gas().into_reth(),
            value: self.value().into_reth(),
            input: self.data().into_reth().into(),
            nonce: self.nonce().into_reth(),
            chain_id: self.chain_id().into_reth(),
            access_list: self.access_list().into_reth(),
            transaction_type: match self {
                EthersTypedTransaction::Legacy(_) => Some(Uint::from(0)),
                EthersTypedTransaction::Eip2930(_) => Some(Uint::from(1)),
                EthersTypedTransaction::Eip1559(_) => Some(Uint::from(2)),
            },
            max_fee_per_blob_gas: None,
            blob_versioned_hashes: None,
        }
    }
}
