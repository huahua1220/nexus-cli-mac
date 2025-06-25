# Nexus Network 0.7.9 证明接口升级说明

## 概述

本次升级保持了0.7.9的核心架构，同时融入了0.8.8版本的证明接口改进，提升了证明效率和错误处理能力。

## 主要改进

### 1. 证明接口优化
- **改进的错误处理**: 添加了`MalformedTask`和`GuestProgram`错误类型
- **退出代码验证**: 集成了`KnownExitCodes`检查，确保证明程序正确退出
- **Task结构体**: 引入了统一的Task结构体，兼容0.8.8接口设计

### 2. 新增功能
- **Task模块**: 新增`src/task.rs`模块，提供任务抽象
- **prove_with_task函数**: 支持Task结构体的证明函数
- **改进的匿名证明**: 更严格的验证和错误处理

### 3. 架构改进
- **保持0.7.9架构**: 维持批量节点管理和节点池替换机制
- **融入0.8.8接口**: 证明逻辑更加健壮和标准化
- **向前兼容**: 旧的证明函数仍然可用

## 技术细节

### 证明函数改进

```rust
// 旧版本
pub fn prove_anonymously() -> Result<(), ProverError>

// 新版本 - 添加了退出代码验证
pub fn prove_anonymously() -> Result<(), ProverError> {
    let (view, _proof) = stwo_prover.prove_with_input::<(), u32>(&(), &public_input)?;
    let exit_code = view.exit_code()?;
    if exit_code != KnownExitCodes::ExitSuccess as u32 {
        return Err(ProverError::GuestProgram(...));
    }
    // ...
}
```

### Task结构体

```rust
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub task_id: String,
    pub program_id: String,
    pub public_inputs: Vec<u8>,
}
```

### 新的证明函数

```rust
// 支持Task结构体的证明函数
pub fn prove_with_task(task: &Task) -> Result<Vec<u8>, ProverError>

// 改进的输入验证
fn get_public_input(task: &Task) -> Result<u32, ProverError>
```

## 依赖更新

- **nexus-sdk**: 更新到tag `0.3.4`，与0.8.8保持一致
- **新导入**: 添加了`KnownExitCodes`和`Viewable`trait

## 使用方式

### 批量挖矿（保持不变）
```bash
./nexus batch-file --file nodes.txt --env beta --max-concurrent 10
```

### 单节点挖矿（保持不变）
```bash
./nexus start --node-id 12345 --env beta
```

## 兼容性

- ✅ **完全向后兼容**: 现有的挖矿脚本和配置文件无需修改
- ✅ **保持0.7.9特性**: 批量节点管理、节点池替换等功能完整保留
- ✅ **增强稳定性**: 更好的错误处理和证明验证

## 注意事项

1. **性能提升**: 新的证明接口提供了更严格的验证，可能略微影响性能
2. **错误信息**: 错误信息更加详细和准确
3. **日志改进**: 证明成功/失败的日志更加清晰

## 技术支持

如果在使用过程中遇到问题，请检查：
1. 节点ID配置是否正确
2. 网络连接是否稳定
3. 日志中的详细错误信息

---

*本升级保持了0.7.9的高效批量挖矿能力，同时提升了证明接口的健壮性和标准化程度。* 