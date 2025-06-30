# Nexus-cli Mac批量版（原版，还能用，新版我在改）
Linux版：https://github.com/huahua1220/nexus-cli-linux

有问题联系推特：https://x.com/hua_web3

Nexus Network CLI 证明器的Mac批量版本。

## 安装

### 1. 安装Rust环境（有的话直接第2步）
```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

source ~/.zshrc

```

### 2. 克隆并编译
```bash
# 克隆并编译
git clone https://github.com/huahua1220/nexus-cli-mac.git
cd nexus-cli-mac
cargo build --release
touch ./target/release/nodes.txt
pwd
```

## 使用方法

### 准备节点文件
上面执行完，然后会显示一个路径，打开一个新文件夹，按着路径进去，再进入 `target/release/nodes.txt` 文件，一行一个节点ID，保存

### 批量模式
```bash
# 运行批量挖矿，如果第二步的终端没关执行下面
./target/release/nexus batch-file --file ./target/release/nodes.txt --max-concurrent 线程数
#如果第二步的终端关了，新开一个终端执行下面
./nexus-cli-mac/target/release/nexus batch-file --file ./nexus-cli-mac/target/release/nodes.txt --max-concurrent 线程数
```

## 主要改进

- 批量节点id启动，防止单个节点id出现429等错误影响效率
