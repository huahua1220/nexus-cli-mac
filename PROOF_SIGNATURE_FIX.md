# Nexus Network 0.7.9 证明签名修复

## 问题描述
原始0.7.9版本在提交证明时出现400错误：
```
Node-XXX: ⚠️ Attempt 1/5 failed: Orchestrator error: Failed to submit proof: [400] Invalid request
```

## 根本原因
Nexus服务器现在要求所有证明提交包含Ed25519数字签名验证，但0.7.9版本的protobuf结构体缺少必要的`ed25519_public_key`和`signature`字段。

## 修复内容

### 1. 更新Protobuf定义
- **文件**: `src/proto/nexus.orchestrator.rs`
- **更改**: 为`GetProofTaskRequest`添加`ed25519_public_key`字段（tag=3）
- **更改**: 为`SubmitProofRequest`添加`ed25519_public_key`字段（tag=7）和`signature`字段（tag=8）

### 2. 密钥管理系统
- **文件**: `src/keys.rs`（新增）
- **功能**: 
  - 自动生成并持久化Ed25519签名密钥到`~/.nexus/node.key`
  - 密钥重用避免重复生成
  - 安全文件权限设置(600)

### 3. 签名验证支持
- **文件**: `src/orchestrator_client.rs`
- **功能**:
  - 所有任务请求自动包含公钥
  - 证明提交包含正确的签名格式
  - 签名消息格式：`"signature_version | task_id | proof_hash"`

### 4. 任务接口升级
- **文件**: `src/task.rs`（新增）
- **功能**: 提供与0.8.8兼容的Task结构体和转换

### 5. 证明函数改进
- **文件**: `src/prover.rs`
- **功能**:
  - 新增`prove_with_task()`函数
  - 增强错误处理（`MalformedTask`、`GuestProgram`等）
  - 集成`KnownExitCodes`验证

## 使用方法

### 编译
```bash
cd 0.7.9
cargo build --release
```

### 单节点模式
```bash
./target/release/nexus start --node-id YOUR_NODE_ID --env devnet
```

### 批量模式
```bash
./target/release/nexus batch-file --file ./nodes.txt --env devnet --proof-interval 3 --start-delay 2 --max-concurrent 10 --verbose
```

## 调试指南

### 1. 检查密钥生成
密钥文件应该存在于：`~/.nexus/node.key`

```bash
ls -la ~/.nexus/node.key
# 应该显示32字节文件，权限为-rw-------
```

### 2. 调试模式测试
创建一个单节点测试：

```bash
# 测试单个节点是否能够正常连接和认证
./target/release/nexus start --node-id 1234567 --env devnet
```

观察输出：
- ✅ 正常：显示任务获取和证明生成
- ❌ 400错误：签名验证失败
- ❌ 连接错误：网络或服务器问题

### 3. 检查签名格式
如果仍然出现400错误，可能的原因：
1. 签名消息格式不匹配
2. 公钥格式问题
3. 服务器端验证逻辑变更

### 4. 网络诊断
如果出现连接错误：
```bash
# 检查与服务器的连接
curl -v https://beta.orchestrator.nexus.xyz/v3/tasks

# 检查DNS解析
nslookup beta.orchestrator.nexus.xyz
```

## 状态码说明
- **200**: 成功
- **400**: 请求格式错误（通常是签名问题）
- **401**: 认证失败
- **429**: 频率限制
- **502/504**: 服务器临时不可用

## 常见问题

### Q: 仍然出现400错误怎么办？
A: 
1. 检查密钥文件是否正确生成
2. 尝试删除`~/.nexus/node.key`让系统重新生成
3. 确认网络连接正常
4. 使用单节点模式测试

### Q: 连接错误如何解决？
A:
1. 检查网络连接
2. 确认防火墙设置
3. 稍后重试（可能是服务器临时维护）

### Q: 如何查看详细错误信息？
A: 使用批量模式的`--verbose`选项可以看到更详细的日志

## 兼容性说明
- ✅ 保持所有0.7.9原有功能
- ✅ 批量节点管理
- ✅ 节点池替换机制  
- ✅ 固定行显示系统
- ✅ 向后兼容现有节点配置文件

## 版本信息
- **修复版本**: 基于0.7.9
- **兼容版本**: 0.8.8 API
- **修复日期**: 2024年当前时间
- **测试状态**: 编译通过，等待用户验证 