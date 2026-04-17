# OS 内核测试指南

本项目包含两条测试路径：

- `os_by_rust`：`no_std` 内核，运行在 QEMU 中（主测试路径）。
- `async_test`：宿主机上的异步执行器演示，不计入内核回归。

## 1. 环境准备（仓库根目录执行）

```bash
rustup toolchain install nightly
rustup default nightly
rustup component add rust-src llvm-tools-preview
cargo install bootimage
cargo bootimage
```

说明：

- `bootimage` 负责构建可启动镜像并通过 runner 拉起 QEMU。
- `.cargo/config.toml` 已配置默认 target 和 runner，以下命令都在仓库根目录执行。
- `rust-toolchain.toml` 已固定 nightly 工具链与必要组件。

## 2. 主工程（内核）测试入口

### 2.1 运行全部内核测试

```bash
cargo test
```

### 2.2 按类型运行

```bash
# 仅运行库内测试（src/lib.rs 与其模块里的 #[test_case]）
cargo test --lib

# 运行指定集成测试
cargo test --test heap_allocation
cargo test --test should_panic
cargo test --test stack_overflow
cargo test --test basic_boot
cargo test --test executor_smoke
```

### 2.3 启动内核（非测试）

```bash
cargo run
```

## 3. async_test（宿主机演示）

该目录用于验证异步调度思路，和内核测试解耦：

```bash
cd async_test
cargo run
```

## 4. 期望结果

- `cargo test` 结束后返回成功状态码。
- `cargo run` 能进入 QEMU 并输出内核日志（含异步任务日志）。
- `async_test/cargo run` 可独立完成示例任务输出。

## 5. 常见问题

### 5.1 `bootimage` 命令不存在

```bash
cargo install bootimage
```

### 5.2 编译时报缺少 `rust-src`

```bash
rustup component add rust-src llvm-tools-preview
```

### 5.3 QEMU 无法启动

- 确认系统已安装 `qemu-system-x86_64`。
- 确认命令在仓库根目录执行，避免找不到 `.cargo/config.toml` 中的 target 配置。
