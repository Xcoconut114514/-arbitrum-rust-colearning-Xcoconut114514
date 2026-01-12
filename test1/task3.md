# Task 3：转账 Gas 费用计算与转账脚本说明

## 代码路径

- 转账脚本：`test1/src/bin/transfer.rs`
- 环境变量模板：`test1/.env.example`

## 如何运行

1. 先做估算（不广播交易）：

   - 保持 `DO_SEND=0`
   - 运行：`cargo run --bin transfer`

2. 真正发送交易（广播到测试网）：

   - 在本地 `test1/.env` 配置 `PRIVATE_KEY`（只用于测试网，且不会提交）
   - 设置 `DO_SEND=1`
   - 运行：`cargo run --bin transfer`

## 脚本的 Gas/费用计算逻辑（代码注释说明）

脚本的目标是：

- 构造一笔普通 ETH 转账交易（`to + value`）
- 先通过 RPC 估算 `gas_limit` 与 `gas_price`
- 计算预计手续费：

$$
\text{estimated\_fee\_wei} = \text{gas\_limit} \times \text{gas\_price\_wei}
$$

并把 `wei` 格式化为 18 位小数的 `ETH` 字符串展示。

### 1) 构造交易请求

- 发送用交易：`tx_for_send`
  - 必含：`to`、`value`、`from`
  - 当 `DO_SEND=1` 时会被签名并广播

- 估算用交易：`tx_for_estimate`
  - 默认等于 `tx_for_send`
  - 但当“未配置 PRIVATE_KEY 且只是 dry-run（DO_SEND=0）”时，会强制把 `value` 设为 0

原因：有些 RPC 节点在 `eth_estimateGas` 时会检查 `from` 的余额是否足够覆盖 `value + gas*price`。
如果没有真实私钥，就只能用一个 dummy 地址作为 `from`，这个地址通常没余额，导致 estimate 直接报错（即使你只是想估算 gas）。

对“普通 ETH 转账”而言，**gas 用量通常固定为 21000**，并不依赖转账金额 `value` 的大小，因此在 dry-run 场景用 `value=0` 来估算 gas 是可行的；手续费的关键在 `gas_limit` 与 `gas_price`。

### 2) 估算 gas_limit

脚本调用：

- `provider.estimate_gas(&tx_for_estimate).await?`

得到 `gas_limit`（例如普通转账常见 `21000`）。

### 3) 获取 gas_price

脚本调用：

- `provider.get_gas_price().await?`

得到 `gas_price_wei`。

说明：这里用的是传统的 `gasPrice` 估算方式（适用于多数 RPC），然后用 `gas_limit * gas_price` 得出预计手续费。

### 4) 计算预计手续费

脚本逻辑：

- `fee_wei = U256::from(gas_limit) * U256::from(gas_price)`
- 打印：`estimated_fee_wei`
- 同时调用 `format_wei_as_eth(fee_wei)` 打印 `estimated_fee_eth`

`format_wei_as_eth` 的实现是：

- 以 $10^{18}$ 为基数把 `wei` 拆成整数部分与小数部分
- 小数部分左侧补 0，固定输出 18 位

### 5) 广播交易与回执

- 当 `DO_SEND=0`：只输出估算结果并退出（不广播）
- 当 `DO_SEND=1`：
  - `provider.send_transaction(tx_for_send).await?`
  - 输出 `tx_hash`
  - `pending.get_receipt().await?` 等待并输出回执（便于截图验收）
