#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        provider::{spawn_http_provider, spawn_ipc_provider},
        *,
    };
    use serial_test::serial;
    use std::{path::Path, time::Duration};

    const TEST_DB_PATH: &str = "/NVMe/data/reth/db";
    const TEST_HTTP_URL: &str = "http://localhost:8489";
    const TEST_IPC_PATH: &str = "/tmp/reth.ipc";

    #[tokio::test]
    #[serial]
    async fn test_get_address() {
        // Create a runtime and handle here for the TaskManager
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();

        let provider = spawn_ipc_provider(TEST_IPC_PATH).await.unwrap();
        let middleware =
            RethMiddleware::new(provider, Path::new(TEST_DB_PATH), handle.clone()).unwrap();

        let ens: NameOrAddress = "vanbeethoven.eth".parse().unwrap();
        let address = middleware.get_address(ens).await.unwrap();
        assert_eq!(address, "0x0e3FfF21A1Cef4f29F7D8cecff3cE4Dfa7703fBc".parse().unwrap());

        rt.shutdown_timeout(Duration::from_secs(0));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_storage_at() {
        // Create a runtime and handle here for the TaskManager
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();

        let provider = spawn_http_provider(TEST_HTTP_URL).await.unwrap();
        let middleware =
            RethMiddleware::new(provider, Path::new(TEST_DB_PATH), handle.clone()).unwrap();

        let from: NameOrAddress = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852".parse().unwrap();
        let location = EthersH256::from_low_u64_be(5);
        let storage = middleware.get_storage_at(from, location, None).await.unwrap();

        // ----------------------------------------------------------- //
        //             Storage slots of UniV2Pair contract             //
        // =========================================================== //
        // storage[5] = factory: address                               //
        // storage[6] = token0: address                                //
        // storage[7] = token1: address                                //
        // storage[8] = (res0, res1, ts): (uint112, uint112, uint32)   //
        // storage[9] = price0CumulativeLast: uint256                  //
        // storage[10] = price1CumulativeLast: uint256                 //
        // storage[11] = kLast: uint256                                //
        // =========================================================== //

        // convert the H256 value to bytes, then take the last 20 bytes and create an address from
        // it
        let decoded_addr = EthersAddress::from_slice(&storage.as_bytes()[12..]);

        let factory_address: NameOrAddress =
            "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f".parse().unwrap();

        if let NameOrAddress::Address(expected_addr) = factory_address {
            assert_eq!(decoded_addr, expected_addr);
        } else {
            panic!("Failed to parse expected factory address");
        }

        rt.shutdown_timeout(Duration::from_secs(0));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_code() {
        // Create a runtime and handle here for the TaskManager
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle();

        let provider = spawn_ipc_provider(TEST_IPC_PATH).await.unwrap();
        let middleware =
            RethMiddleware::new(provider, Path::new(TEST_DB_PATH), handle.clone()).unwrap();
        let address: NameOrAddress = "0x0e3FfF21A1Cef4f29F7D8cecff3cE4Dfa7703fBc".parse().unwrap();
        let code = middleware.get_code(address, None).await.unwrap();
        // Address contains no code
        assert_eq!(code.len(), 0);

        rt.shutdown_timeout(Duration::from_secs(0));
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
