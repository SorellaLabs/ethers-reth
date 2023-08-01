#[cfg(test)]
mod test_utils;

// All testdata for this was generated with reth node running on goerli
// with manually set growth step to 4 kB in reth_db::implementation::mdbx::mod.rs:81
//
// `growth_step: Some(4 * 1024 as isize)`
//
// Command:
// `reth node --chain goerli --datadir ./testdata --http --http.api all --debug.tip
// 0xe9006d7148f879e1af79d12ba532d061e160ded8f9066c3d74c9724f65366d94`
mod tests {
    use std::path::{Path, PathBuf};

    use ethers::{
        prelude::k256::ecdsa::SigningKey,
        providers::Middleware,
        signers::Wallet,
        types::{
            transaction::{
                eip2718::TypedTransaction as EthersTypedTransaction,
                eip2930::AccessListWithGasUsed as EthersAccessListWithGasUsed,
            },
            Address as EthersAddress, Block as EthersBlock, BlockId as EthersBlockId,
            BlockNumber as EthersBlockNumber, BlockTrace as EthersBlockTrace, Bytes as EthersBytes,
            Eip1559TransactionRequest, FeeHistory as EthersFeeHistory, Filter as EthersFilter,
            FilterBlockOption as EthersFilterBlockOption, GethTrace as EthersGethTrace,
            Log as EthersLog, NameOrAddress as EthersNameOrAddress, Trace as EthersTrace,
            TraceType as EthersTraceType, Transaction as EthersTransaction,
            TransactionReceipt as EthersTransactionReceipt,
            TransactionRequest as EthersTransactionRequest, TxHash as EthersTxHash,
            H256 as EthersH256, U256 as EthersU256,
        },
    };

    use reth_primitives::{DEV, MAINNET, U64};

    use serial_test::serial;

    use pretty_assertions::assert_eq;

    use crate::test_utils::{spawn_reth_middleware, MAINNET_HTTP_URL};

    const BLOCK_NUMBER: u64 = 3;
    const BLOCK_HASH: &str = "0x5ba8efbbe87e1f50f06bea90637e360bcf126e38d37d3d31bd0e5ed62d37fc7b";

    const WALLET_ADDRESS: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    const WETH_ADDRESS: &str = "0xe7f1725e7734ce288f8367e1bb143e90bb3f0512";
    const WETH_DEPLOY_TX_HASH: &str =
        "0x0b6fd363c5d835f29518d686a362972ac0fcb13552a5f95633354d4d0226b50f";

    fn get_testdata_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
    }

    fn get_db_dir() -> PathBuf {
        get_testdata_dir().join("db")
    }

    #[tokio::test]
    #[serial]
    async fn test_get_address() {
        // We have to use the mainnet here because the goerli testnet does not have an ENS resolver
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, MAINNET.clone(), get_db_dir()).await;

        let ens: EthersNameOrAddress = "vanbeethoven.eth".parse().unwrap();
        let address = reth_middleware.get_address(ens).await.unwrap();
        assert_eq!(
            address,
            "0x0e3FfF21A1Cef4f29F7D8cecff3cE4Dfa7703fBc".parse().unwrap(),
            "Address {} does not match expected address",
            address
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_call() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_id: EthersBlockId = BLOCK_NUMBER.into();
        let erc20_address: EthersNameOrAddress = WETH_ADDRESS.into();
        let call_data: EthersBytes =
            "0x70a08231000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"
                .parse()
                .unwrap();
        let call_transaction: &EthersTypedTransaction = &EthersTypedTransaction::Eip1559(
            Eip1559TransactionRequest::new().to(erc20_address).data(call_data),
        );
        let call_result = reth_middleware.call(call_transaction, Some(block_id)).await.unwrap();

        let expected_call_result: EthersBytes =
            "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000".parse().unwrap();

        assert_eq!(expected_call_result, call_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_estimate_gas() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_id: EthersBlockId = BLOCK_NUMBER.into();
        let from: EthersAddress = WALLET_ADDRESS.parse().unwrap();
        let to: EthersNameOrAddress = WETH_ADDRESS.into();

        let call_data: EthersBytes =
            "0x70a08231000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"
                .parse()
                .unwrap();
        let call_transaction: &EthersTypedTransaction = &EthersTypedTransaction::Eip1559(
            Eip1559TransactionRequest::new().from(from).to(to).data(call_data),
        );

        let estimate_gas =
            reth_middleware.estimate_gas(call_transaction, Some(block_id)).await.unwrap();

        let expected_estimate_gas: EthersU256 = "0x5d9e".parse().unwrap();

        assert_eq!(expected_estimate_gas, estimate_gas);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_access_list() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_id: EthersBlockId = BLOCK_NUMBER.into();
        let from: EthersAddress = WALLET_ADDRESS.parse().unwrap();
        let to: EthersNameOrAddress = WETH_ADDRESS.into();

        let call_data: EthersBytes =
            "0x70a08231000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"
                .parse()
                .unwrap();
        let call_transaction: &EthersTypedTransaction = &EthersTypedTransaction::Eip1559(
            Eip1559TransactionRequest::new().from(from).to(to).data(call_data),
        );

        let access_list =
            reth_middleware.create_access_list(call_transaction, Some(block_id)).await.unwrap();

        let expected_access_list_path = get_testdata_dir().join("expected_access_list.json");
        let expected_access_list = serde_json::from_str::<EthersAccessListWithGasUsed>(
            &std::fs::read_to_string(expected_access_list_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_access_list.access_list, access_list.access_list);
        assert_eq!(expected_access_list.gas_used, access_list.gas_used);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_storage_at() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let address: EthersNameOrAddress = WETH_ADDRESS.into();
        let location: EthersH256 =
            "0x0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap();
        let block_id: EthersBlockId = BLOCK_NUMBER.into();

        let storage =
            reth_middleware.get_storage_at(address, location, Some(block_id)).await.unwrap();

        let expected_storage: EthersH256 =
            "0x5745544800000000000000000000000000000000000000000000000000000008".parse().unwrap();

        assert_eq!(expected_storage, storage);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_code() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let address: EthersAddress = WETH_ADDRESS.parse().unwrap();
        let block_id: EthersBlockId = BLOCK_NUMBER.into();

        let code = reth_middleware.get_code(address, Some(block_id)).await.unwrap();

        let expected_code_path = get_testdata_dir().join("expected_code.json");
        let expected_code: EthersBytes = serde_json::from_str::<EthersBytes>(
            &std::fs::read_to_string(expected_code_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_code, code);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_balance() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let address: EthersAddress = WALLET_ADDRESS.parse().unwrap();
        let block_id: EthersBlockId = BLOCK_NUMBER.into();

        let balance = reth_middleware.get_balance(address, Some(block_id)).await.unwrap();

        let expected_balance: EthersU256 = "0xd3c20de2a676955968ef".parse().unwrap();

        assert_eq!(expected_balance, balance);
    }

    // eth_getProof is not implemented
    // See https://github.com/paradigmxyz/reth/blob/dbafe23cce45917a43f6640d71a0216fe3268428/crates/rpc/rpc/src/eth/api/server.rs#L377

    #[tokio::test]
    #[serial]
    async fn test_fee_history() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let block_count: EthersU256 = 3.into();
        let reward_percentiles: &[f64] = &[25.0, 75.0];

        let fee_history = reth_middleware
            .fee_history(block_count, block_number, reward_percentiles)
            .await
            .unwrap();

        let expected_fee_history_path: PathBuf =
            get_testdata_dir().join("expected_fee_history.json");
        let expected_fee_history: EthersFeeHistory =
            serde_json::from_str(&std::fs::read_to_string(expected_fee_history_path).unwrap())
                .unwrap();

        assert_eq!(expected_fee_history.base_fee_per_gas, fee_history.base_fee_per_gas);
        assert_eq!(expected_fee_history.gas_used_ratio, fee_history.gas_used_ratio);
        assert_eq!(expected_fee_history.oldest_block, fee_history.oldest_block);
        assert_eq!(expected_fee_history.reward, fee_history.reward);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_chainid() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let chainid = reth_middleware.get_chainid().await.unwrap();

        let expected_chainid: EthersU256 = DEV.clone().chain().id().into();

        assert_eq!(expected_chainid, chainid);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_block_number() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number = reth_middleware.get_block_number().await.unwrap();

        let expected_block_number: U64 = 5.into();

        assert_eq!(expected_block_number, block_number);
    }

    // eth_getBlockReceipts is being deprecated on mainnet and is not available on goerli on
    // Alchemy // https://docs.alchemy.com/reference/eth-getblockreceipts
    #[tokio::test]
    #[serial]
    async fn test_get_block_receipts() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let block_receipts = reth_middleware.get_block_receipts(block_number).await.unwrap();

        let expected_block_receipts_path = get_testdata_dir().join("expected_block_receipts.json");
        let expected_block_receipts: Vec<EthersTransactionReceipt> =
            serde_json::from_str(&std::fs::read_to_string(expected_block_receipts_path).unwrap())
                .unwrap();

        assert_eq!(expected_block_receipts, block_receipts);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_transaction() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let transaction_hash: EthersH256 = WETH_DEPLOY_TX_HASH.parse().unwrap();
        let transaction = reth_middleware.get_transaction(transaction_hash).await.unwrap().unwrap();

        let expected_transaction_path = get_testdata_dir().join("expected_transaction.json");
        let expected_transaction: EthersTransaction =
            serde_json::from_str(&std::fs::read_to_string(expected_transaction_path).unwrap())
                .unwrap();

        assert_eq!(expected_transaction, transaction);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_transaction_receipt() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let transaction_hash: EthersH256 = WETH_DEPLOY_TX_HASH.parse().unwrap();
        let transaction_receipt =
            reth_middleware.get_transaction_receipt(transaction_hash).await.unwrap().unwrap();

        let expected_transaction_receipt_path =
            get_testdata_dir().join("expected_transaction_receipt.json");
        let expected_transaction_receipt: EthersTransactionReceipt = serde_json::from_str(
            &std::fs::read_to_string(expected_transaction_receipt_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_transaction_receipt, transaction_receipt);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_transaction_count() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let transaction_count =
            reth_middleware.get_transaction_count(WALLET_ADDRESS, None).await.unwrap();
        let expected_transaction_count: EthersU256 = 4.into();

        assert_eq!(expected_transaction_count, transaction_count);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_block() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let block = reth_middleware.get_block(block_number).await.unwrap().unwrap();

        let expected_block_path = get_testdata_dir().join("expected_block.json");
        let expected_block: EthersBlock<EthersTxHash> =
            serde_json::from_str(&std::fs::read_to_string(expected_block_path).unwrap()).unwrap();

        assert_eq!(expected_block, block);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_block_with_txs() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let block = reth_middleware.get_block_with_txs(block_number).await.unwrap().unwrap();

        let expected_block_with_txs_path = get_testdata_dir().join("expected_block_with_txs.json");
        let expected_block_with_txs: EthersBlock<EthersTransaction> =
            serde_json::from_str(&std::fs::read_to_string(expected_block_with_txs_path).unwrap())
                .unwrap();

        assert_eq!(expected_block_with_txs, block);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_logs() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_hash: EthersH256 = BLOCK_HASH.parse().unwrap();
        let filter_block_option = EthersFilterBlockOption::AtBlockHash(block_hash);
        let address: EthersAddress = WETH_ADDRESS.parse().unwrap();
        let filter = EthersFilter::new().select(filter_block_option).address(vec![address]);

        let logs = reth_middleware.get_logs(&filter).await.unwrap();

        let expected_logs_path = get_testdata_dir().join("expected_logs.json");
        let expected_logs: Vec<EthersLog> =
            serde_json::from_str(&std::fs::read_to_string(expected_logs_path).unwrap()).unwrap();

        assert_eq!(expected_logs, logs);
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_call() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let from: EthersAddress = WALLET_ADDRESS.parse().unwrap();
        let to: EthersAddress = WETH_ADDRESS.parse().unwrap();
        let call_data: EthersBytes =
            "0xa9059cbb0000000000000000000000006000eca38b8b5bba64986182fe2a69c57f6b541400000000000000000000000000000000000000000000010f0cf064dd59200000"
                .parse()
                .unwrap();
        let call_transaction: EthersTypedTransaction = EthersTypedTransaction::Eip1559(
            Eip1559TransactionRequest::new().from(from).to(to).data(call_data),
        );

        let trace_types =
            vec![EthersTraceType::Trace, EthersTraceType::VmTrace, EthersTraceType::StateDiff];

        let trace_call_result = reth_middleware
            .trace_call(call_transaction, trace_types, Some(block_number))
            .await
            .unwrap();

        let expected_trace_call_path = get_testdata_dir().join("expected_trace_call.json");
        let _ = std::fs::write(
            &expected_trace_call_path,
            serde_json::to_string_pretty(&trace_call_result).unwrap(),
        );
        let expected_trace_call: EthersBlockTrace =
            serde_json::from_str(&std::fs::read_to_string(expected_trace_call_path).unwrap())
                .unwrap();
        assert_eq!(expected_trace_call, trace_call_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_call_many() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let from: EthersAddress = WALLET_ADDRESS.parse().unwrap();
        let to: EthersAddress = WETH_ADDRESS.parse().unwrap();

        let call_data_transfer: EthersBytes =
            "0xa9059cbb0000000000000000000000006000eca38b8b5bba64986182fe2a69c57f6b541400000000000000000000000000000000000000000000010f0cf064dd59200000"
                .parse()
                .unwrap();
        let call_data_approve: EthersBytes =
            "0x095ea7b30000000000000000000000001dc4c1cefef38a777b15aa20260a54e584b16c48ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                .parse()
                .unwrap();
        let call_transaction_transfer: EthersTypedTransaction = EthersTypedTransaction::Eip1559(
            Eip1559TransactionRequest::new().from(from).to(to).data(call_data_transfer),
        );
        let call_transaction_approve: EthersTypedTransaction = EthersTypedTransaction::Eip1559(
            Eip1559TransactionRequest::new().from(from).to(to).data(call_data_approve),
        );

        let trace_types =
            vec![EthersTraceType::Trace, EthersTraceType::VmTrace, EthersTraceType::StateDiff];

        let trace_call_many_result = reth_middleware
            .trace_call_many(
                vec![
                    (call_transaction_transfer, trace_types.clone()),
                    (call_transaction_approve, trace_types.clone()),
                ],
                Some(block_number),
            )
            .await
            .unwrap();

        let expected_trace_call_many_path =
            get_testdata_dir().join("expected_trace_call_many.json");
        let _ = std::fs::write(
            &expected_trace_call_many_path,
            serde_json::to_string_pretty(&trace_call_many_result).unwrap(),
        );
        let expected_trace_call_many: Vec<EthersBlockTrace> =
            serde_json::from_str(&std::fs::read_to_string(expected_trace_call_many_path).unwrap())
                .unwrap();
        assert_eq!(expected_trace_call_many, trace_call_many_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_raw_transaction() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let wallet: Wallet<SigningKey> =
            "0000000000000000000000000000000000000000000000000000000000000001".parse().unwrap();
        let from: EthersAddress = WALLET_ADDRESS.parse().unwrap();
        let to: EthersAddress = WETH_ADDRESS.parse().unwrap();
        let call_data: EthersBytes =
            "0x70a08231000000000000000000000000871dd7c2b4b25e1aa18728e9d5f2af4c4e431f5c"
                .parse()
                .unwrap();
        let transaction: EthersTypedTransaction = EthersTypedTransaction::Legacy(
            EthersTransactionRequest::new()
                .from(from)
                .to(to)
                .data(call_data)
                .chain_id(DEV.clone().chain().id())
                .gas(100000),
        );
        let signature = wallet.sign_transaction_sync(&transaction).unwrap();
        let raw_transaction = transaction.rlp_signed(&signature);

        let trace_types =
            vec![EthersTraceType::Trace, EthersTraceType::VmTrace, EthersTraceType::StateDiff];

        let trace_raw_transaction_result =
            reth_middleware.trace_raw_transaction(raw_transaction, trace_types).await.unwrap();

        let expected_trace_raw_transaction_path = get_testdata_dir()
            .join("expected_trace_raw_transaction.json")
            .to_str()
            .unwrap()
            .to_string();
        let _ = std::fs::write(
            &expected_trace_raw_transaction_path,
            serde_json::to_string_pretty(&trace_raw_transaction_result).unwrap(),
        );
        let expected_trace_raw_transaction: EthersBlockTrace = serde_json::from_str(
            &std::fs::read_to_string(expected_trace_raw_transaction_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_trace_raw_transaction, trace_raw_transaction_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_replay_transaction() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let transaction_hash: EthersH256 = WETH_DEPLOY_TX_HASH.parse().unwrap();

        let trace_types =
            vec![EthersTraceType::Trace, EthersTraceType::VmTrace, EthersTraceType::StateDiff];
        let trace_replay_transaction_result: EthersBlockTrace =
            reth_middleware.trace_replay_transaction(transaction_hash, trace_types).await.unwrap();

        let expected_trace_replay_transaction_path =
            get_testdata_dir().join("expected_trace_replay_transaction.json");
        let _ = std::fs::write(
            &expected_trace_replay_transaction_path,
            serde_json::to_string_pretty(&trace_replay_transaction_result).unwrap(),
        );
        let expected_trace_replay_transaction: EthersBlockTrace = serde_json::from_str(
            &std::fs::read_to_string(expected_trace_replay_transaction_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_trace_replay_transaction, trace_replay_transaction_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_replay_block_transactions() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();

        let trace_types =
            vec![EthersTraceType::Trace, EthersTraceType::VmTrace, EthersTraceType::StateDiff];
        let trace_replay_block_transactions_result = reth_middleware
            .trace_replay_block_transactions(block_number, trace_types)
            .await
            .unwrap();

        let expected_trace_replay_block_transactions_path =
            get_testdata_dir().join("expected_trace_replay_block_transactions.json");
        let _ = std::fs::write(
            &expected_trace_replay_block_transactions_path,
            serde_json::to_string_pretty(&trace_replay_block_transactions_result).unwrap(),
        );
        let expected_trace_replay_block_transactions: Vec<EthersBlockTrace> = serde_json::from_str(
            &std::fs::read_to_string(expected_trace_replay_block_transactions_path).unwrap(),
        )
        .unwrap();

        assert_eq!(
            expected_trace_replay_block_transactions,
            trace_replay_block_transactions_result
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_block() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let trace_block_result = reth_middleware.trace_block(block_number).await.unwrap();

        let expected_trace_block_path = get_testdata_dir().join("expected_trace_block.json");
        let _ = std::fs::write(
            expected_trace_block_path.clone(),
            serde_json::to_string_pretty(&trace_block_result).unwrap(),
        );
        let expected_trace_block: Vec<EthersTrace> =
            serde_json::from_str(&std::fs::read_to_string(expected_trace_block_path).unwrap())
                .unwrap();

        assert_eq!(expected_trace_block, trace_block_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_get() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let transaction_hash: EthersH256 = WETH_DEPLOY_TX_HASH.parse().unwrap();
        let trace_get_result = reth_middleware.trace_get(transaction_hash, vec![0]).await.unwrap();

        let expected_trace_get_path = get_testdata_dir().join("expected_trace_get.json");
        let expected_trace_get: EthersTrace =
            serde_json::from_str(&std::fs::read_to_string(expected_trace_get_path).unwrap())
                .unwrap();

        assert_eq!(expected_trace_get, trace_get_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_trace_transaction() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let transaction_hash: EthersH256 = WETH_DEPLOY_TX_HASH.parse().unwrap();

        let trace_transaction_result =
            reth_middleware.trace_transaction(transaction_hash).await.unwrap();

        let expected_trace_transaction_path =
            get_testdata_dir().join("expected_trace_transaction.json");
        let expected_trace_transaction: Vec<EthersTrace> = serde_json::from_str(
            &std::fs::read_to_string(expected_trace_transaction_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_trace_transaction, trace_transaction_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_debug_trace_transaction() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let transaction_hash: EthersH256 = WETH_DEPLOY_TX_HASH.parse().unwrap();
        let debug_trace_transaction_result = reth_middleware
            .debug_trace_transaction(transaction_hash, Default::default())
            .await
            .unwrap();

        let expected_debug_trace_transaction_path =
            get_testdata_dir().join("expected_debug_trace_transaction.json");
        let expected_debug_trace_transaction: EthersGethTrace = serde_json::from_str(
            &std::fs::read_to_string(expected_debug_trace_transaction_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_debug_trace_transaction, debug_trace_transaction_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_debug_trace_block_by_hash() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_hash: EthersH256 = BLOCK_HASH.parse().unwrap();

        let debug_trace_block_by_hash_result = reth_middleware
            .debug_trace_block_by_hash(block_hash, Default::default())
            .await
            .unwrap();

        let expected_debug_trace_block_by_hash_path =
            get_testdata_dir().join("expected_debug_trace_block_by_hash.json");
        let expected_debug_trace_block_by_hash: Vec<EthersGethTrace> = serde_json::from_str(
            &std::fs::read_to_string(expected_debug_trace_block_by_hash_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_debug_trace_block_by_hash, debug_trace_block_by_hash_result);
    }

    #[tokio::test]
    #[serial]
    async fn test_debug_trace_block_by_number() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_number: EthersBlockNumber = BLOCK_NUMBER.into();
        let debug_trace_block_by_number_result = reth_middleware
            .debug_trace_block_by_number(Some(block_number), Default::default())
            .await
            .unwrap();

        let expected_debug_trace_block_by_number_path =
            get_testdata_dir().join("expected_debug_trace_block_by_number.json");
        let expected_debug_trace_block_by_number: Vec<EthersGethTrace> = serde_json::from_str(
            &std::fs::read_to_string(expected_debug_trace_block_by_number_path).unwrap(),
        )
        .unwrap();

        assert_eq!(expected_debug_trace_block_by_number, debug_trace_block_by_number_result);
    }

    #[tokio::test]
    #[serial]
    async fn debug_trace_call() {
        let reth_middleware =
            spawn_reth_middleware(MAINNET_HTTP_URL, DEV.clone(), get_db_dir()).await;

        let block_id: EthersBlockId = BLOCK_NUMBER.into();
        let from: EthersAddress = WALLET_ADDRESS.parse().unwrap();
        let to: EthersNameOrAddress = WETH_ADDRESS.into();

        let call_data: EthersBytes =
            "0xa9059cbb0000000000000000000000006000eca38b8b5bba64986182fe2a69c57f6b541400000000000000000000000000000000000000000000010f0cf064dd59200000"
                .parse()
                .unwrap();
        let call_transaction: EthersTypedTransaction = EthersTypedTransaction::Eip1559(
            Eip1559TransactionRequest::new().from(from).to(to).data(call_data).gas(1000000),
        );
        let debug_trace_call_result = reth_middleware
            .debug_trace_call(call_transaction, Some(block_id), Default::default())
            .await
            .unwrap();

        let expected_debug_trace_call_path =
            get_testdata_dir().join("expected_debug_trace_call.json");
        let expected_debug_trace_call: EthersGethTrace =
            serde_json::from_str(&std::fs::read_to_string(expected_debug_trace_call_path).unwrap())
                .unwrap();

        assert_eq!(expected_debug_trace_call, debug_trace_call_result);
    }
}
