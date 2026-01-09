use alloy::network::EthereumWallet;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use eyre::Result;

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
        return Err(eyre::eyre!("地址长度不对：期望 40 位 hex（不含 0x），实际为 {}：{}", s.len(), input));
    }
    let bytes = alloy::hex::decode(s)?;
    Ok(Address::from_slice(&bytes))
}

fn parse_eth_amount_to_wei(input: &str) -> Result<U256> {
    // 只支持最多 18 位小数的十进制字符串，例如 0.0015
    let s = input.trim();
    let (whole, frac) = match s.split_once('.') {
        Some((a, b)) => (a, b),
        None => (s, ""),
    };

    let whole: U256 = whole.parse()?;
    let mut frac_str = frac.to_string();
    if frac_str.len() > 18 {
        return Err(eyre::eyre!("小数位最多 18 位：{}", input));
    }
    while frac_str.len() < 18 {
        frac_str.push('0');
    }
    let frac: U256 = if frac_str.is_empty() { U256::ZERO } else { frac_str.parse()? };

    let base = U256::from(10u64).pow(U256::from(18u64));
    Ok(whole * base + frac)
}

#[tokio::main]
async fn main() -> Result<()> {
    // 优先读取本地 .env（不提交），如果不存在则读取 .env.example（仅用于非敏感配置/示例）。
    dotenvy::from_filename(".env").ok();
    dotenvy::from_filename(".env.example").ok();

    let rpc_url = std::env::var("RPC_URL")
        .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());

    let do_send = std::env::var("DO_SEND")
        .map(|v| v.trim() == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    // 目标地址：按你的要求默认转到这个地址，也可用 env 覆盖。
    let to = std::env::var("TO_ADDRESS")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(|v| parse_address(&v))
        .transpose()?
        .unwrap_or_else(|| parse_address("0xa7413b9E079127ce7F202c4b132df18204F52436").unwrap());

    // 默认转 0.0001 ETH（测试网）。
    let amount_eth = std::env::var("AMOUNT_ETH").unwrap_or_else(|_| "0.0001".to_string());
    let value = parse_eth_amount_to_wei(&amount_eth)?;

    println!("to: {to}");
    println!("amount_eth: {amount_eth}");
    println!("value_wei: {value}");

    // provider：为了避免类型分支不兼容，这里始终构造带 wallet filler 的 provider。
    // 当未配置 PRIVATE_KEY 且 DO_SEND=0 时，使用固定 dummy signer 仅用于估算 gas（不会广播）。
    let private_key = std::env::var("PRIVATE_KEY").ok().filter(|v| !v.trim().is_empty());
    let has_private_key = private_key.is_some();
    if do_send && private_key.is_none() {
        return Err(eyre::eyre!(
            "DO_SEND=1 需要 PRIVATE_KEY（仅测试网）。请在 test1/.env 中配置 PRIVATE_KEY。"
        ));
    }

    let signer: PrivateKeySigner = match private_key {
        Some(pk) => pk.parse()?,
        None => "0x0101010101010101010101010101010101010101010101010101010101010101".parse()?,
    };
    let from = signer.address();
    if has_private_key {
        println!("from: {from}");
    } else {
        println!("from: {from} (dummy, estimation only)");
    }

    let wallet = EthereumWallet::from(signer);
    let provider = ProviderBuilder::new().wallet(wallet).on_http(rpc_url.parse()?);

    let chain_id = provider.get_chain_id().await?;
    println!("chain_id: {chain_id}");

    // 组装交易请求（由 provider 的 fillers 自动补全 nonce、gas、fee 并签名；estimate 时不会广播）。
    // 注意：部分节点会在 estimateGas 时检查 from 是否有足够余额覆盖 `value`。
    // 因此在未配置 PRIVATE_KEY 的 dry-run 场景下，用 value=0 来估算 gas（gas 用量与普通转账一致）。
    let tx_for_send = TransactionRequest::default().to(to).value(value).from(from);
    let tx_for_estimate = if !has_private_key && !do_send {
        println!("note: estimating with value=0 (no PRIVATE_KEY configured); gas fee is still representative for a plain ETH transfer");
        TransactionRequest::default().to(to).value(U256::ZERO).from(from)
    } else {
        tx_for_send.clone()
    };

    // 估算 gas 与 gasPrice，计算预计费用。
    let gas_limit = provider.estimate_gas(&tx_for_estimate).await?;
    let gas_price = provider.get_gas_price().await?;

    let fee_wei = U256::from(gas_limit) * U256::from(gas_price);
    println!("gas_limit: {gas_limit}");
    println!("gas_price_wei: {gas_price}");
    println!("estimated_fee_wei: {fee_wei}");
    println!("estimated_fee_eth: {}", format_wei_as_eth(fee_wei));

    if !do_send {
        println!("DO_SEND is not enabled; dry-run only. Set DO_SEND=1 to broadcast the transaction.");
        return Ok(());
    }

    let pending = provider.send_transaction(tx_for_send).await?;
    // 明确输出 tx hash，并等待回执，便于截图/验收。
    let tx_hash = pending.tx_hash();
    println!("tx_hash: {tx_hash:?}");

    println!("waiting for receipt...");
    let receipt = pending.get_receipt().await?;
    println!("receipt: {receipt:?}");

    Ok(())
}
