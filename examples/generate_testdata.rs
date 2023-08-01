use std::sync::Arc;

use ethers::{
    prelude::{Contract, ContractFactory, SignerMiddleware},
    providers::{Http, Middleware, Provider},
    signers::{coins_bip39::English, LocalWallet, MnemonicBuilder, Signer},
    types::{
        transaction::eip2718::TypedTransaction, Address, TransactionReceipt, TransactionRequest,
    },
    utils::parse_ether,
};
use eyre::Result;

use crate::contracts::weth9::{WETH9, WETH9_ABI, WETH9_BYTECODE};

mod contracts;

// Wallet address: 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
// Ethereum transfer hash 0x2b4e87a87401818db7b9c5023ce3912e7caf19cbd1d1b8591a8236e213eb5056
// Contract deployment tx hash: 0x0b6fd363c5d835f29518d686a362972ac0fcb13552a5f95633354d4d0226b50f
// Contract address: 0xe7f1725e7734ce288f8367e1bb143e90bb3f0512
// Deposit tx hash: 0x8e1b2cee99bc5a68111574f76f883bf6a0b7d727279f3f40d594ef017048d394

#[tokio::main]
async fn main() -> Result<()> {
    // 0. Generate the contract bindings from the ABI
    // Abigen::new("WETH9", "./examples/contracts/WETH9.json")
    //     .expect("could not instantiate Abigen")
    //     .generate()
    //     .expect("could not generate bindings")
    //     .write_module_in_dir("./examples/contracts");

    // 1. Initialize a new Http provider for local reth node
    let rpc_url = "http://localhost:8545";
    let provider =
        Provider::<Http>::try_from(rpc_url).expect("Could not instantiate HTTP Provider");

    // 2. Initialize a new wallet
    let wallet = MnemonicBuilder::<English>::default()
        .phrase("test test test test test test test test test test test junk")
        .index(0u32)?
        .build()?;
    let wallet: LocalWallet = wallet.with_chain_id(provider.get_chainid().await?.as_u64());
    let sender_addr = wallet.address();
    println!("Wallet address: {:?}", sender_addr);

    // 3. Instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, wallet.clone());
    let client = Arc::new(client);

    // Check wallet balance
    let _balance = client.get_balance(sender_addr, None).await?;

    // Send 1 eth to address(0)
    let tx: TypedTransaction = TransactionRequest {
        from: Some(sender_addr),
        to: Some("F0109fC8DF283027b6285cc889F5aA624EaC1F55".parse::<Address>().unwrap().into()),
        value: Some(1_000_000_000u64.into()),
        ..Default::default()
    }
    .into();
    let pending_tx = client.send_transaction(tx, None).await?;

    let receipt = pending_tx.await?.ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    println!("Ethereum transfer hash {:?}", receipt.transaction_hash);

    // 5. Deploy contract
    let factory = ContractFactory::new(WETH9_ABI.clone(), WETH9_BYTECODE.clone(), client.clone());
    let (contract, receipt): (Contract<_>, TransactionReceipt) =
        factory.deploy(())?.send_with_receipt().await?;
    println!("Contract deployment tx hash: {:?}", receipt.transaction_hash);
    println!("Contract address: {:?}", contract.address());

    let weth: WETH9<_> = contract.into();

    // 6. Deposit eth
    let deposit_receipt = weth
        .deposit()
        .value(parse_ether(1).unwrap())
        .send()
        .await?
        .await?
        .expect("transaction failed");
    println!("Deposit tx hash: {:?}", deposit_receipt.transaction_hash);

    Ok(())
}
