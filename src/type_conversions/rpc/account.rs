use crate::type_conversions::{ToEthers, ToReth};

use ethers::types::{
    EIP1186ProofResponse as EthersEIP1186ProofResponse, StorageProof as EthersStorageProof,
};
use reth_primitives::serde_helper::JsonStorageKey;

use reth_rpc_types::{EIP1186AccountProofResponse, StorageProof};

/// TransactionReceipt (ethers) -> (reth)
impl ToReth<StorageProof> for EthersStorageProof {
    fn into_reth(self) -> StorageProof {
        StorageProof {
            key: JsonStorageKey(self.key.into_reth()),
            value: self.value.into_reth(),
            proof: self.proof.into_reth(),
        }
    }
}
/// TransactionReceipt (reth) -> (ethers)
impl ToEthers<EthersStorageProof> for StorageProof {
    fn into_ethers(self) -> EthersStorageProof {
        EthersStorageProof {
            key: self.key.0.into_ethers(),
            value: self.value.into_ethers(),
            proof: self.proof.into_ethers(),
        }
    }
}

// -----------------------------------------------

/// EIP1186AccountProofResponse (ethers) -> (reth)
impl ToReth<EIP1186AccountProofResponse> for EthersEIP1186ProofResponse {
    fn into_reth(self) -> EIP1186AccountProofResponse {
        EIP1186AccountProofResponse {
            address: self.address.into_reth(),
            balance: self.balance.into_reth(),
            code_hash: self.code_hash.into_reth(),
            nonce: self.nonce.into_reth(),
            storage_hash: self.storage_hash.into_reth(),
            account_proof: self.account_proof.into_reth(),
            storage_proof: self.storage_proof.into_reth(),
        }
    }
}

/// EIP1186AccountProofResponse (reth) -> (ethers)
impl ToEthers<EthersEIP1186ProofResponse> for EIP1186AccountProofResponse {
    fn into_ethers(self) -> EthersEIP1186ProofResponse {
        EthersEIP1186ProofResponse {
            address: self.address.into_ethers(),
            balance: self.balance.into_ethers(),
            code_hash: self.code_hash.into_ethers(),
            nonce: self.nonce.into_ethers(),
            storage_hash: self.storage_hash.into_ethers(),
            account_proof: self.account_proof.into_ethers(),
            storage_proof: self.storage_proof.into_ethers(),
        }
    }
}
