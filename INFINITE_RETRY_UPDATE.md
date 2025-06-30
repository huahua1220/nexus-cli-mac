# Nexus Network 0.7.9 无限重试机制更新

## 更新内容

根据用户要求，已将**节点自动替换机制**改为**无限重试机制**。节点在失败时不再被替换，而是持续重试直到成功。

## 主要修改

### 1. 移除节点替换逻辑 ❌
- **删除**: `NodePoolManager` 结构体和相关功能
- **删除**: `replace_failed_node()` 方法
- **删除**: `spawn_replacement_node()` 函数
- **删除**: 节点池管理相关代码

### 2. 实现无限重试机制 ♾️
- **修改**: `run_authenticated_proving_loop_optimized()` - 移除最大失败次数限制
- **修改**: `run_anonymous_proving_loop_optimized()` - 移除最大失败次数限制  
- **修改**: `run_authenticated_proving_loop_with_callback()` - 支持无限重试
- **修改**: `run_anonymous_proving_loop_with_callback()` - 支持无限重试

### 3. 优化重试策略 🔄
- **一般错误**: 失败后等待5-10秒再重试
- **429限流错误**: 等待60秒后重试（而不是终止节点）
- **显示格式**: 从 `(failure X/Y)` 改为 `(retry X/∞)`

### 4. 简化批量处理 📊
- **移除**: 复杂的节点池管理
- **改为**: 简单的并发处理，启动指定数量的节点
- **未使用节点**: 显示未使用的节点数量供参考

## 使用方法

### 编译
```bash
cd 0.7.9
cargo build --release
```

### 批量挖矿（推荐）
```bash
./target/release/nexus batch-file --file ./nodes.txt --env devnet --max-concurrent 10 --proof-interval 3 --start-delay 2
```

### 单节点挖矿
```bash
./target/release/nexus start --node-id YOUR_NODE_ID --env devnet
```

## 行为变化

### 之前（自动替换模式）：
- 节点失败3-5次后被自动替换
- 从节点池中取新节点继续运行
- 失败节点被永久移除

### 现在（无限重试模式）：
- 节点永不被替换
- 失败后等待片刻继续重试
- 所有节点持续运行直到手动停止

## 实际效果

### 成功输出示例：
```
Node-1234567: [10:15:30] ✅ Proof #5 completed successfully
Node-1234568: [10:15:32] ✅ Proof #3 completed successfully
```

### 失败重试示例：
```
Node-1234567: [10:15:35] ❌ Proof #6 failed: Orchestrator error (retry 1/∞)
Node-1234567: [10:15:36] 🔄 Waiting 10s before retry...
Node-1234567: [10:15:46] ⚠️ Attempt 1/5 failed: Connection timeout
Node-1234567: [10:15:48] 🔄 Retrying in 2s...
```

### 限流处理示例：
```
Node-1234567: [10:15:50] 🚫 Rate limited: [429] Too many requests - waiting 60s
Node-1234567: [10:16:50] ⚠️ Attempt 3/5 failed: Rate limited
```

## 优势

### ✅ 优点：
1. **持续挖矿**: 节点永不停止，最大化挖矿时间
2. **简化管理**: 不需要复杂的节点池管理
3. **网络适应**: 自动适应网络波动和临时故障
4. **资源利用**: 充分利用所有可用节点

### ⚠️ 注意事项：
1. **手动停止**: 需要手动按 Ctrl+C 停止程序
2. **资源消耗**: 持续重试可能增加网络请求
3. **日志较多**: 失败重试会产生更多日志输出

## 配置建议

### 并发数量：
- **小型部署**: `--max-concurrent 5-10`
- **中型部署**: `--max-concurrent 10-20` 
- **大型部署**: `--max-concurrent 20-50`

### 时间间隔：
- **证明间隔**: `--proof-interval 3-5` (秒)
- **启动延迟**: `--start-delay 1-3` (秒)

## 兼容性

- ✅ **向后兼容**: 保持所有原有命令行选项
- ✅ **节点列表**: 兼容现有的节点列表文件格式
- ✅ **配置文件**: 兼容现有的配置文件
- ✅ **环境选项**: 支持所有环境 (devnet, testnet, mainnet)

## 故障排除

### 如果节点频繁失败：
1. 检查网络连接稳定性
2. 确认节点ID有效性
3. 降低并发数量
4. 增加证明间隔时间

### 如果出现大量429错误：
1. 降低并发数量
2. 增加证明间隔
3. 检查是否有其他程序同时运行

---

**重要**: 此更新将挖矿模式从"替换失败节点"改为"永不放弃重试"，确保最大化的挖矿持续时间和成功率。 