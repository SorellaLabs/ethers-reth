use eyre::Context;
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};

use crate::{RethApi, RethDebug, RethFilter, RethMiddleware, RethTrace};
use ethers::providers::Middleware;
// Reth
use reth_db::{
    database::{Database, DatabaseGAT},
    mdbx::{Env, WriteMap},
    tables,
    transaction::DbTx,
    DatabaseError,
};
use reth_network_api::noop::NoopNetwork;
use reth_primitives::{constants::ETHEREUM_BLOCK_GAS_LIMIT, MAINNET};
use reth_provider::{providers::BlockchainProvider, ProviderFactory};
use reth_revm::Factory;
use reth_rpc::{
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    DebugApi, EthApi, EthFilter, TraceApi, TracingCallGuard, TracingCallPool,
};
use reth_tasks::TaskManager;
use reth_transaction_pool::{EthTransactionValidator, CoinbaseTipOrdering, Pool, PooledTransaction};
// Std
use std::{fmt::Debug, path::Path, sync::Arc};
use tokio::runtime::Handle;

pub type Provider = BlockchainProvider<
    Arc<Env<WriteMap>>,
    ShareableBlockchainTree<Arc<Env<WriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type RethTxPool =
    Pool<EthTransactionValidator<Provider, PooledTransaction>, CoinbaseTipOrdering<PooledTransaction>>;

impl<M> RethMiddleware<M>
where
    M: Middleware,
{
    pub fn try_new(
        db_path: &Path,
        handle: Handle,
    ) -> Result<(RethApi, RethFilter, RethTrace, RethDebug), DatabaseError> {
        let task_manager = TaskManager::new(handle);
        let task_executor = task_manager.executor();

        tokio::task::spawn(task_manager);

        let chain = MAINNET.clone();
        let db = Arc::new(init_db(db_path).unwrap());

        let tree_externals = TreeExternals::new(
            db.clone(),
            Arc::new(BeaconConsensus::new(Arc::clone(&chain))),
            Factory::new(chain.clone()),
            Arc::clone(&chain),
        );

        let tree_config = BlockchainTreeConfig::default();

        let (canon_state_notification_sender, _receiver) =
            tokio::sync::broadcast::channel(tree_config.max_reorg_depth() as usize * 2);

        let blockchain_tree = ShareableBlockchainTree::new(
            BlockchainTree::new(tree_externals, canon_state_notification_sender, tree_config)
                .unwrap(),
        );

        let provider = BlockchainProvider::new(
            ProviderFactory::new(Arc::clone(&db), Arc::clone(&chain)),
            blockchain_tree,
        )
        .unwrap();

        let state_cache = EthStateCache::spawn(provider.clone(), EthStateCacheConfig::default());

        let tx_pool = reth_transaction_pool::Pool::eth_pool(
            EthTransactionValidator::new(provider.clone(), chain, task_executor.clone()),
            Default::default(),
        );

        let reth_api = EthApi::new(
            provider.clone(),
            tx_pool.clone(),
            NoopNetwork::default(),
            state_cache.clone(),
            GasPriceOracle::new(
                provider.clone(),
                GasPriceOracleConfig::default(),
                state_cache.clone(),
            ),
            ETHEREUM_BLOCK_GAS_LIMIT,
            TracingCallPool::build().unwrap(),
        );

        let tracing_call_guard = TracingCallGuard::new(10);

        let reth_trace =
            TraceApi::new(provider.clone(), reth_api.clone(), tracing_call_guard.clone());

        let reth_debug = DebugApi::new(
            provider.clone(),
            reth_api.clone(),
            Box::new(task_executor.clone()),
            tracing_call_guard,
        );

        let reth_filter =
            EthFilter::new(provider, tx_pool, state_cache, 1000, Box::new(task_executor));

        Ok((reth_api, reth_filter, reth_trace, reth_debug))
    }
}

/// re-implementation of 'view()'
/// allows for a function to be passed in through a RO libmdbx transaction
/// /reth/crates/storage/db/src/abstraction/database.rs
pub fn view<F, T>(db: &Env<WriteMap>, f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&<Env<WriteMap> as DatabaseGAT<'_>>::TX) -> T,
{
    let tx = db.tx()?;
    let res = f(&tx);
    tx.commit()?;

    Ok(res)
}

/// Opens up an existing database at the specified path.
pub fn init_db<P: AsRef<Path> + Debug>(path: P) -> eyre::Result<Env<WriteMap>> {
    let _ = std::fs::create_dir_all(path.as_ref());
    let db = reth_db::mdbx::Env::<reth_db::mdbx::WriteMap>::open(
        path.as_ref(),
        reth_db::mdbx::EnvKind::RO,
        None,
    )?;

    view(&db, |tx| {
        for table in tables::Tables::ALL.iter().map(|table| table.name()) {
            tx.inner.open_db(Some(table)).wrap_err("Could not open db.").unwrap();
        }
    })?;

    Ok(db)
}
