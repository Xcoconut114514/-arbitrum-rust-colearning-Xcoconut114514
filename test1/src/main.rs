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

alloy::sol! {
    #[sol(rpc)]
    interface Counter {
        function number() external view returns (uint256);
        function increment() external;
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    // 只读 Provider：不需要私钥，也能查询链信息、调用 view 函数。
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    // 可选：如果提供了 PRIVATE_KEY，则构造可签名 Provider，用于发送交易。
    let signing_provider = match std::env::var("PRIVATE_KEY") {
        Ok(private_key) if !private_key.trim().is_empty() => {
            let signer: PrivateKeySigner = private_key.parse()?;
            let address = signer.address();
            println!("wallet_address: {address}");

            let wallet = EthereumWallet::from(signer);
            Some(ProviderBuilder::new().wallet(wallet).on_http(rpc_url.parse()?))
        }
        _ => {
            println!("wallet_address: (not set)");
            None
        }
    };

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

    if let Ok(contract_addr) = std::env::var("COUNTER_CONTRACT") {
        let contract_addr = contract_addr.parse()?;
        let counter = Counter::new(contract_addr, provider.clone());

        let number = counter.number().call().await?;
        println!("counter.number(): {}", number._0);

        let do_increment = std::env::var("DO_INCREMENT")
            .map(|v| v.trim() == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        if do_increment {
            let signing_provider = signing_provider.ok_or_else(|| {
                eyre::eyre!(
                    "DO_INCREMENT=1 需要 PRIVATE_KEY。请在 .env 中填入测试网私钥，并确保账户有测试 ETH。"
                )
            })?;

            let counter = Counter::new(contract_addr, signing_provider);
            let pending = counter.increment().send().await?;
            println!("increment tx sent: {pending:?}");
        }
    }

    Ok(())
}
