use crate::defaults::{default_client, default_pool, NodeClient, NodeTxPool};
use async_trait::async_trait;
use ethers::prelude::Client;
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
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    EthApi,
};
use reth_transaction_pool::{CostOrdering, EthTransactionValidator, Pool, PooledTransaction};
use std::{path::Path, sync::Arc};

pub type NodeEthApi = EthApi<NodeClient, NodeTxPool, NoopNetwork>;

/// default EthApi implementation
pub fn default_eth_api(client: NodeClient) -> NodeEthApi {
    let tx_pool = default_pool(client.clone());

    let state_cache = EthStateCache::spawn(client.clone(), EthStateCacheConfig::default());

    let api = EthApi::new(
        client.clone(),
        tx_pool,
        NoopNetwork,
        state_cache,
        GasPriceOracle::new(client.clone(), GasPriceOracleConfig::default(), state_cache.clone()),
    );

    api
}
