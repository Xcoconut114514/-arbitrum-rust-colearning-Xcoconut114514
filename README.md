# arbitrum-rust-colearning-xcoconut114514

面向 Arbitrum Sepolia 测试网的 Rust 链上交互练习项目：使用 Alloy + tokio 读取 chain id、最新区块号，并可选调用示例合约 `HelloWeb3.hello_web3()`。

## 1. 前期准备

### 1) 创建 GitHub 仓库

在你的 GitHub 主页创建仓库：

- 仓库名：`arbitrum-rust-colearning-Xcoconut114514`
- 建议勾选：Add a README（也可不勾，本项目已提供 README）

备注：我无法替你在网页上点击创建，但我已把本地项目准备好，下面提供推送命令。

### 2) 安装钱包

- 安装 MetaMask 浏览器插件

### 3) 配置网络（Arbitrum Sepolia）

- 通过 Chainlist 一键添加 `Arbitrum Sepolia` 到 MetaMask
- 关键参数（供核对）：
  - Chain ID：`421614`
  - RPC（本项目默认）：`https://sepolia-rollup.arbitrum.io/rpc`

### 4) 获取测试币（测试网 ETH）

- 使用 Alchemy Faucet 给你的地址领取测试 ETH（用于支付 gas）

## 2. 开发环境搭建（Windows）

### 安装 Rust

安装并切到稳定版：

1. 安装 rustup（官方安装器）
2. 执行：

```powershell
rustup default stable
rustc -V
cargo -V
```

如果你已经装过 Rust，但本地提示找不到 `cargo`，一般是还没装 rustup 或 PATH 未生效（重开终端即可）。

## 3. 合约与链上交互

### 示例合约

示例合约在 [contracts/HelloWeb3.sol](contracts/HelloWeb3.sol)，函数：

- `hello_web3()` -> 返回字符串 `"hello Web3"`

你可以用 Remix 或 Foundry/Hardhat 部署到 Arbitrum Sepolia，然后把部署后的合约地址填到 `.env` 的 `HELLO_CONTRACT`。

### 示例交易（参考）

本任务描述中提到“示例交易哈希和合约地址可在 Arbiscan 查看”。这里不硬编码任何 hash（避免过期/错误）。

你可以：

- 在 Arbiscan 的 Arbitrum Sepolia 页面里搜索你自己的交易 hash
- 或在 MetaMask 的活动记录里复制交易 hash

## 4. Rust 项目实践

### 依赖

本项目使用：

- `alloy`：Rust Web3/以太坊工具库
- `tokio`：异步运行时

参考资源：

- Alloy 文档：https://alloy.rs
- Alloy GitHub：https://github.com/alloy-rs/alloy

### 运行

1) 准备环境变量：

```powershell
Copy-Item .env.example .env
```

2) 编辑 `.env`：填入 `PRIVATE_KEY`（仅测试网）以及可选的 `HELLO_CONTRACT`。

3) 运行：

```powershell
cargo run
```

预期输出包括：

- `wallet_address: 0x...`
- `chain_id: 421614`
- `latest_block: ...`
- 如果配置了 `HELLO_CONTRACT`，还会输出 `hello_web3(): hello Web3`

## 5. 最终任务：推送到 GitHub

在本目录执行（PowerShell）：

```powershell
git init
git add -A
git commit -m "init: arbitrum rust co-learning"

# 把下面 URL 换成你刚创建的仓库地址
git branch -M main
git remote add origin https://github.com/Xcoconut114514/arbitrum-rust-colearning-Xcoconut114514.git
git push -u origin main
```

如果你更喜欢命令行创建仓库（可选，需要安装 GitHub CLI 并登录）：

```powershell
gh repo create Xcoconut114514/arbitrum-rust-colearning-Xcoconut114514 --public --source . --remote origin --push
```
