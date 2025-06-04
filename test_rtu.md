# Modbus RTU 测试指南

这个文档描述了如何测试新实现的Modbus RTU功能，展示了应用层和网络层的清晰分离。

## 架构概述

我们的设计采用了分层架构，实现了应用层和网络层的清晰分离：

### 1. 应用层 (Application Layer)
- **责任**: 业务逻辑、数据解释、设备管理
- **文件**: `src/bin/rtu_test.rs` 中的 `EmsApplication`
- **功能**:
  - 设备配置管理
  - 寄存器映射定义
  - 数据采集逻辑
  - 数据格式转换（如IEEE 754浮点数）
  - 错误处理和重试逻辑

### 2. 协议层 (Protocol Layer)
- **责任**: Modbus协议实现、功能码处理
- **文件**: `src/protocol.rs`
- **功能**:
  - Modbus功能码定义
  - 请求/响应结构
  - 数据类型转换
  - 异常处理

### 3. 传输层 (Transport Layer)  
- **责任**: RTU特定的网络通信
- **文件**: `src/transport.rs` 中的 `RtuTransport`
- **功能**:
  - 串口通信
  - RTU帧编码/解码
  - CRC校验
  - 超时处理
  - 连接管理

## 测试程序

### 1. RTU功能测试 (`rtu_test`)

这个程序演示了分层架构的使用：

```bash
# 编译项目
cargo build --release

# 运行RTU测试（需要连接RTU设备）
cargo run --bin rtu_test

# 设置环境变量来配置串口
MODBUS_PORT=/dev/ttyUSB0 MODBUS_BAUD=9600 cargo run --bin rtu_test
```

**测试内容**:
- 网络层测试：直接测试RTU传输层功能
- 应用层测试：模拟EMS系统的数据采集流程

### 2. RTU模拟器 (`rtu_simulator`)

当没有真实硬件时，可以使用模拟器进行测试：

```bash
# 运行模拟器
cargo run --bin rtu_simulator

# 查看模拟器输出
RUST_LOG=debug cargo run --bin rtu_simulator
```

## 硬件测试设置

### 1. 使用真实串口设备

如果您有Modbus RTU设备：

```bash
# 在Linux上查看可用串口
ls /dev/ttyUSB* /dev/ttyACM*

# 检查串口权限
sudo chmod 666 /dev/ttyUSB0

# 运行测试
MODBUS_PORT=/dev/ttyUSB0 MODBUS_BAUD=9600 cargo run --bin rtu_test
```

### 2. 使用虚拟串口对

如果没有真实硬件，可以创建虚拟串口对：

```bash
# 安装socat工具
sudo apt-get install socat  # Ubuntu/Debian
brew install socat          # macOS

# 创建虚拟串口对
socat -d -d pty,raw,echo=0 pty,raw,echo=0

# 这会创建两个虚拟串口，例如：
# /dev/pts/1 和 /dev/pts/2

# 在一个终端运行模拟器（修改为使用虚拟串口）
# 在另一个终端运行测试客户端
```

## 测试场景

### 1. 基本功能测试

测试所有基本的Modbus功能：

- **读取保持寄存器** (0x03)
- **读取输入寄存器** (0x04)  
- **读取线圈** (0x01)
- **读取离散输入** (0x02)
- **写单个寄存器** (0x06)
- **写单个线圈** (0x05)
- **写多个寄存器** (0x10)
- **写多个线圈** (0x0F)

### 2. 错误处理测试

- 无效从站ID
- 无效寄存器地址
- CRC错误
- 超时处理
- 异常响应

### 3. 性能测试

- 高频率请求
- 大批量数据传输
- 并发请求处理
- 内存使用监控

### 4. 应用层测试

模拟真实的EMS（能源管理系统）场景：

- 多设备数据采集
- 浮点数据处理
- 设备状态监控
- 数据时间戳记录

## 配置选项

### 串口配置

```rust
let transport = RtuTransport::new_with_config(
    "/dev/ttyUSB0",      // 串口路径
    9600,                // 波特率
    DataBits::Eight,     // 数据位
    StopBits::One,       // 停止位
    Parity::None,        // 校验位
    Duration::from_millis(1000), // 超时
)?;
```

### 应用层配置

```rust
let device_config = DeviceConfig {
    slave_id: 1,
    name: "Inverter-1".to_string(),
    device_type: "Solar Inverter".to_string(),
    register_map: RegisterMap {
        voltage_registers: vec![0x0000, 0x0001, 0x0002],
        current_registers: vec![0x0010, 0x0011, 0x0012],
        power_registers: vec![0x0020, 0x0021],
        status_coils: vec![0x0000, 0x0001, 0x0002],
    },
};
```

## 架构优势

### 1. 模块化设计
- 各层职责清晰，便于维护
- 可以独立测试每一层
- 容易扩展和修改

### 2. 可测试性
- 网络层可以使用模拟器测试
- 应用层可以使用Mock传输层测试
- 协议层有独立的单元测试

### 3. 可重用性
- 传输层可以支持不同的应用
- 应用层可以使用不同的传输层（TCP/RTU）
- 协议层在所有传输方式中通用

### 4. 错误处理
- 每层都有适当的错误处理
- 错误信息包含足够的上下文
- 支持错误链和错误分类

## 日志和调试

设置不同的日志级别来调试问题：

```bash
# 基本信息
RUST_LOG=info cargo run --bin rtu_test

# 详细调试信息
RUST_LOG=debug cargo run --bin rtu_test

# 传输层调试
RUST_LOG=voltage_modbus::transport=trace cargo run --bin rtu_test

# 协议层调试  
RUST_LOG=voltage_modbus::protocol=trace cargo run --bin rtu_test
```

## 性能监控

程序内置了性能统计功能：

- 请求发送数量
- 响应接收数量
- 错误和超时计数
- 传输字节统计
- 成功率计算

## 扩展建议

### 1. 增加更多设备类型
- 定义新的设备配置模板
- 实现设备特定的数据解析逻辑

### 2. 实现配置文件支持
- YAML/JSON配置文件
- 热重载配置
- 设备发现功能

### 3. 添加Web界面
- 实时数据显示
- 设备状态监控
- 历史数据查询

### 4. 数据库集成
- 历史数据存储
- 趋势分析
- 报警管理

这个测试框架展示了如何通过清晰的架构分离来构建可维护、可测试的工业通信系统。 