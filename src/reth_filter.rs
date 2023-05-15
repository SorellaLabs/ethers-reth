
use async_trait::async_trait;
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};
use reth_db::mdbx::{Env, EnvKind, NoWriteMap};
use reth_network_api::test_utils::NoopNetwork;
use reth_primitives::MAINNET;
use reth_provider::{providers::BlockchainProvider, ShareableDatabase};
use reth_revm::Factory;
use reth_rpc::{
    eth::{cache::{EthStateCache, EthStateCacheConfig}, gas_oracle::{GasPriceOracle, GasPriceOracleConfig}},
    EthApi, EthFilter,
};
use reth_transaction_pool::{EthTransactionValidator, Pool, PooledTransaction, CostOrdering};
use std::{sync::Arc, path::Path};

use crate::{defaults::{default_client, default_pool, NodeClient, NodeTxPool}};


pub type NodeEthFilter = EthFilter<NodeClient, NodeTxPool>;


/// default EthFilter implementation
pub fn default_eth_filter(client: NodeClient, max_logs_per_response: usize) -> NodeEthFilter {

    let tx_pool = default_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let filter = EthFilter::new(
        client, 
        tx_pool, 
        state_cache, 
        max_logs_per_response
    );
    
    filter
}


