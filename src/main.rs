use alloy::network::EthereumWallet;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use dotenvy::dotenv;
use eyre::Result;

alloy::sol! {
    #[sol(rpc)]
    interface HelloWeb3 {
        function hello_web3() external view returns (string);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    let private_key = std::env::var("PRIVATE_KEY").map_err(|_| {
        eyre::eyre!(
            "缺少环境变量 PRIVATE_KEY。请复制 .env.example 为 .env 并填入你的私钥（仅测试网）。"
        )
    })?;

    let signer: PrivateKeySigner = private_key.parse()?;
    let address = signer.address();
    println!("wallet_address: {address}");

    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(rpc_url.parse()?);

    let chain_id = provider.get_chain_id().await?;
    println!("chain_id: {chain_id}");

    let latest_block = provider.get_block_number().await?;
    println!("latest_block: {latest_block}");

    if let Ok(contract_addr) = std::env::var("HELLO_CONTRACT") {
        let contract_addr = contract_addr.parse()?;
        let contract = HelloWeb3::new(contract_addr, provider.clone());
        let response = contract.hello_web3().call().await?;
        println!("hello_web3(): {}", response._0);
    }

    Ok(())
}
