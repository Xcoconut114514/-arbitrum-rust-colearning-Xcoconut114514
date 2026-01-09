use alloy::network::EthereumWallet;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::{Address, U256};
use alloy::signers::local::PrivateKeySigner;
use eyre::Result;
use serde_json::json;

fn format_wei_as_eth(wei: U256) -> String {
    let base = U256::from(10u64).pow(U256::from(18u64));
    let whole = wei / base;
    let frac = wei % base;
    format!("{whole}.{:018}", frac)
}

fn parse_address(input: &str) -> Result<Address> {
    let s = input.trim().trim_matches('"');
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.len() != 40 {
        return Err(eyre::eyre!(
            "地址长度不对：期望 40 位 hex（不含 0x），实际为 {}：{}",
            s.len(),
            input
        ));
    }
    let bytes = alloy::hex::decode(s)?;
    Ok(Address::from_slice(&bytes))
}

async fn get_balance_wei_lenient(rpc_url: &str, address: Address) -> Result<U256> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getBalance",
        "params": [format!("{address:#x}"), "latest"],
    });

    let value: serde_json::Value = reqwest::Client::new()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if let Some(err) = value.get("error") {
        return Err(eyre::eyre!("RPC error: {err}"));
    }

    let result = value
        .get("result")
        .and_then(|v| v.as_str())
        .ok_or_else(|| eyre::eyre!("RPC response missing result: {value}"))?;

    // 一些节点会返回奇数长度的 hex quantity（例如 0x55d...），这里做容错。
    let hex = result.trim().strip_prefix("0x").unwrap_or(result.trim());
    if hex.is_empty() {
        return Ok(U256::ZERO);
    }
    let normalized = if hex.len() % 2 == 1 {
        format!("0{hex}")
    } else {
        hex.to_string()
    };
    let bytes = alloy::hex::decode(normalized)?;
    Ok(U256::from_be_slice(&bytes))
}

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
    // 优先读取本地 .env（不提交），如果不存在则读取 .env.example（仅用于非敏感配置/示例）。
    dotenvy::from_filename(".env").ok();
    dotenvy::from_filename(".env.example").ok();

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    // 只读 Provider：不需要私钥，也能查询链信息、调用 view 函数。
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    // 可选：如果提供了 PRIVATE_KEY，则构造可签名 Provider，用于发送交易。
    let mut wallet_address: Option<Address> = None;
    let signing_provider = match std::env::var("PRIVATE_KEY") {
        Ok(private_key) if !private_key.trim().is_empty() => {
            let signer: PrivateKeySigner = private_key.parse()?;
            let address = signer.address();
            wallet_address = Some(address);
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

    // 查询余额：优先使用 BALANCE_ADDRESS，否则（若存在 PRIVATE_KEY）查询当前钱包地址。
    let balance_address = match std::env::var("BALANCE_ADDRESS") {
        Ok(v) if !v.trim().is_empty() => Some(parse_address(&v)?),
        _ => wallet_address,
    };

    if let Some(addr) = balance_address {
        let balance_wei = get_balance_wei_lenient(&rpc_url, addr).await?;
        println!("balance_address: {addr}");
        println!("balance_wei: {}", balance_wei);
        println!("balance_eth: {}", format_wei_as_eth(balance_wei));
    }

    if let Ok(contract_addr) = std::env::var("HELLO_CONTRACT") {
        if !contract_addr.trim().is_empty() {
            let contract_addr = parse_address(&contract_addr)?;
            let contract = HelloWeb3::new(contract_addr, provider.clone());
            let response = contract.hello_web3().call().await?;
            println!("hello_web3(): {}", response._0);
        }
    }

    if let Ok(contract_addr) = std::env::var("COUNTER_CONTRACT") {
        if contract_addr.trim().is_empty() {
            // 未配置合约地址则跳过
        } else {
            let contract_addr = parse_address(&contract_addr)?;
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
    }

    Ok(())
}
