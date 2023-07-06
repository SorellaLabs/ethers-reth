#[cfg(test)]
mod test_utils;

mod tests {
    use crate::test_utils::{init_testdata, spawn_http_provider, TestDb};

    use ethers::{
        prelude::Lazy,
        providers::{Http, Middleware, Provider},
        types::{Bytes as EthersBytes, NameOrAddress, H256 as EthersH256},
    };
    use ethers_reth::{type_conversions::ToEthers, RethMiddleware};
    use serial_test::serial;

    // const TEST_HTTP_URL: &str = "https://reth.sorella-beechit.com:8485";
    const TEST_HTTP_URL: &str = "http://45.250.253.77:8545";
    // const TEST_HTTP_URL: &str = "https://eth.llamarpc.com";
    #[allow(dead_code)]
    const TEST_IPC_PATH: &str = "/tmp/reth.ipc";

    static TEST_DB: Lazy<TestDb> = Lazy::new(|| init_testdata());

    #[tokio::test]
    #[serial]
    async fn test_get_address() {
        // Create a runtime and handle here for the TaskManager
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();

        // let provider = spawn_ipc_provider(TEST_IPC_PATH).await.expect(format!("Failed to spawn
        // IPC provider from path {}", TEST_IPC_PATH).as_str());
        let provider = spawn_http_provider(TEST_HTTP_URL)
            .await
            .expect(format!("Failed to spawn HTTP provider from path {}", TEST_HTTP_URL).as_str());

        let block_number = provider.get_block_number().await.unwrap();
        println!("block_number: {:?}", block_number);

        let middleware = RethMiddleware::new(provider, &TEST_DB.path, handle.clone()).unwrap();

        let ens: NameOrAddress = "vanbeethoven.eth".parse().unwrap();
        let address = middleware.get_address(ens).await.unwrap();
        assert_eq!(address, "0x0e3FfF21A1Cef4f29F7D8cecff3cE4Dfa7703fBc".parse().unwrap());

        rt.shutdown_background();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_storage_at() {
        // Create a runtime and handle here for the TaskManager
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();

        let provider = spawn_http_provider(TEST_HTTP_URL).await.unwrap();
        let middleware = RethMiddleware::new(provider, &TEST_DB.path, handle.clone()).unwrap();

        for (addr, (_account, storages)) in TEST_DB.state.iter() {
            for entry in storages.iter() {
                let storage = middleware
                    .get_storage_at(
                        NameOrAddress::Address((*addr).into()),
                        entry.key.into_ethers(),
                        None,
                    )
                    .await
                    .unwrap();
                let expected_storage = entry.value;
                let expected_storage =
                    EthersH256::from_slice(&expected_storage.to_be_bytes::<32>());
                assert_eq!(storage, expected_storage);
            }
        }

        rt.shutdown_background();
    }

    #[tokio::test]
    #[serial]
    async fn test_get_code() {
        // Create a runtime and handle here for the TaskManager
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();

        let provider = spawn_http_provider(TEST_HTTP_URL).await.unwrap();
        let middleware = RethMiddleware::new(provider, &TEST_DB.path, handle.clone()).unwrap();

        for (addr, bytecode) in &TEST_DB.bytecodes {
            let code: EthersBytes =
                middleware.get_code(NameOrAddress::Address((*addr).into()), None).await.unwrap();
            let expected_code: EthersBytes = bytecode.bytecode.to_vec().into();
            assert_eq!(expected_code, code);
        }

        rt.shutdown_background();
    }
}

//TODO:

// Create DB integration tests using the reth test db
// Create the reth middleware & understand how to create stub components of each
// During re-read through of the code I should keep an eye out for:
// - db test env
/* - pub type RethClient = BlockchainProvider<
    Arc<Env<WriteMap>>,
    ShareableBlockchainTree<Arc<Env<WriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type RethTxPool =
    Pool<EthTransactionValidator<RethClient, PooledTransaction>, CostOrdering<PooledTransaction>>;

pub type RethApi = EthApi<RethClient, RethTxPool, NoopNetwork>;
pub type RethFilter = EthFilter<RethClient, RethTxPool>;
pub type RethTrace = TraceApi<RethClient, RethApi>;

#[derive(Clone)]
pub struct RethMiddleware<M> {
    inner: M,
    reth_api: RethApi,
    reth_filter: RethFilter,
    reth_trace: RethTrace,
} */
