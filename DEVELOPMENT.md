# Music Player 项目文档

## 1. 项目概述

一个基于 Unix Socket 通信的本地音乐播放器，采用 C/S 架构：
- **musicd**: 后台守护进程，负责音频播放和队列管理
- **musicc**: 命令行客户端，通过 Unix Domain Socket 与 daemon 通信

## 2. 架构设计

### 2.1 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                      musicd (Daemon)                        │
│  ┌─────────────┐    ┌──────────────┐    ┌───────────────┐  │
│  │   Queue     │◄──►│  Handlers    │◄──►│ AudioEngine   │  │
│  │  (state)    │    │ (commands)   │    │   (rodio)     │  │
│  └─────────────┘    └──────────────┘    └───────────────┘  │
│         ▲                                        ▲          │
│         │          ┌────────────────┐           │            │
│         └──────────►  Unix Socket  ◄───────────┘            │
│                    │   Listener    │                        │
│                    └────────────────┘                        │
└─────────────────────────────────────────────────────────────┘
                          ▲
                          │ Unix Domain Socket
                          │ (musicd.sock)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    musicc (CLI Client)                      │
│  ┌─────────────┐    ┌──────────────┐    ┌───────────────┐  │
│  │    CLI      │───►│ to_command() │───►│ SocketClient │  │
│  │  (clap)     │    │ (convert)    │    │  (connect)   │  │
│  └─────────────┘    └──────────────┘    └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 组件说明

| 组件 | 包 | 职责 |
|------|-----|------|
| musicd | `musicd/` | 后台服务进程，监听 socket，处理命令 |
| musicc | `musicc/` | CLI 客户端，解析用户输入，发送命令 |
| music-common | `music-common/` | 共享类型和 socket 客户端 |

### 2.3 模块划分

```
music-common/src/
├── lib.rs          # 共享类型定义
│   ├── Command     # 命令枚举（Play, Pause, Add, List 等）
│   ├── Response    # 响应结构（ok, error, body）
│   ├── Queue       # 播放队列（含 current, repeat, shuffle）
│   ├── PlaybackState # 播放状态（Stopped, Playing, Paused）
│   └── RepeatMode  # 循环模式（Off, One, All）
└── socket.rs       # SocketClient 实现

musicd/src/
├── main.rs         # 入口，socket 监听循环
├── handlers.rs     # Command -> Response 处理
├── audio.rs        # rodio 音频播放封装
└── state.rs        # PlayerState + Queue 管理

musicc/src/
└── main.rs         # CLI 入口，clap 解析，socket 通信
```

## 3. 通信协议

### 3.1 Socket 路径
- **路径**: `~/Library/Application Support/music/musicd.sock`
- **类型**: Unix Domain Socket（文件式，非 TCP）

### 3.2 协议格式
基于 JSON 行协议（JSON Lines）：
- 请求: `{"cmd":"play"}\n`
- 响应: `{"id":1,"ok":true,"body":null}\n`

### 3.3 消息格式

**Command (客户端 → 服务端)**:
```json
{"cmd":"play"}
{"cmd":"add","path":"/absolute/path/to/file.mp3"}
{"cmd":"list"}
```

**Response (服务端 → 客户端)**:
```json
{"id":1,"ok":true,"error":null,"body":null}
{"id":1,"ok":true,"error":null,"body":{"items":[...],"current":0,"repeat":"off","shuffle":false}}
```

## 4. 数据结构

### 4.1 Command 枚举
```rust
pub enum Command {
    Play,
    Pause,
    Stop,
    Next,
    Prev,
    Add { path: String },
    Remove { index: usize },
    Move { from: usize, to: usize },
    Clear,
    List,
    Shuffle,
    Repeat { mode: RepeatMode },
    Quit,
    Status,
}
```

### 4.2 Response 结构
```rust
pub struct Response {
    pub id: u64,
    pub ok: bool,
    pub error: Option<String>,
    pub body: Option<serde_json::Value>,
}
```

### 4.3 Queue 结构
```rust
pub struct Queue {
    pub items: Vec<QueueItem>,     // 曲目列表
    pub current: Option<usize>,   // 当前播放索引
    pub repeat: RepeatMode,        // 循环模式
    pub shuffle: bool,             // 随机播放
}
```

### 4.4 PlaybackState 枚举
```rust
pub enum PlaybackState {
    Stopped,
    Playing { track_index: usize },
    Paused { track_index: usize },
}
```

## 5. 持久化

### 5.1 配置文件目录
- **路径**: `~/Library/Application Support/music/`
- **来源**: `dirs::config_dir()` 返回（macOS 不同于此前的 `~/.config/music`）

### 5.2 持久化文件
| 文件 | 内容 |
|------|------|
| `musicd.sock` | Unix Domain Socket |
| `musicd.pid` | 进程 ID |
| `queue.json` | 播放队列（自动保存） |

## 6. 问题修复记录

### 6.1 musicc 命令卡住无响应
**症状**: 执行 `musicc status` 后命令挂起，无输出

**根因**: `SocketClient::connect()` 使用 `read_to_end()` 读取响应。当 musicd 处理完请求关闭连接时，macOS 上 `read_to_end()` 无法正确检测 EOF，一直等待。

**解决方案**: 改用 `BufReader::read_line()` 基于行的读取。
```rust
// 旧代码（有问题）
stream.read_to_end(&mut buf).await.unwrap();

// 新代码（正确）
let mut reader = BufReader::new(stream);
reader.read_line(&mut line).await.unwrap();
```

**涉及文件**: `music-common/src/socket.rs`, `musicd/src/main.rs`

### 6.2 添加文件后播放无声音
**症状**: `musicc add audio_cache/file.mp3` 成功，但 `musicc play` 无声音

**根因**: 用户传入相对路径，musicd 工作目录不确定，无法找到文件。

**解决方案**: musicc 在发送 `Add` 命令前将相对路径转为绝对路径。
```rust
Commands::Add { path } => {
    let absolute = std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(path);
    Command::Add { path: absolute }
}
```

**涉及文件**: `musicc/src/main.rs`

### 6.3 命名冲突
**问题**: macOS 系统自带 Apple Music，路径 `music/socket` 可能冲突

**解决方案**: 重命名文件和 PID
- Socket: `music/socket` → `music/musicd.sock`
- PID: `daemon.pid` → `musicd.pid`

## 7. 命令行接口

### 7.1 musicc 命令

```bash
# 无参数时显示帮助
./target/debug/musicc

# 播放控制
./target/debug/musicc play           # 播放当前曲目或队列下一首
./target/debug/musicc pause          # 暂停
./target/debug/musicc stop           # 停止
./target/debug/musicc next           # 下一首
./target/debug/musicc prev           # 上一首

# 队列管理
./target/debug/musicc add <path>      # 添加文件（自动转绝对路径）
./target/debug/musicc remove <index>  # 移除指定曲目
./target/debug/musicc move <from> <to> # 移动曲目位置
./target/debug/musicc clear          # 清空队列
./target/debug/musicc list           # 查看队列
./target/debug/musicc shuffle         # 随机打乱队列

# 设置
./target/debug/musicc repeat <mode>   # 循环模式: off, one, all
./target/debug/musicc status          # 查看播放状态

# 退出
./target/debug/musicc quit            # 关闭后台服务
```

### 7.2 musicd 命令

```bash
# 启动后台服务（无需参数）
./target/debug/musicd
```

## 8. 依赖

```toml
# workspace 依赖
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rodio = "0.19"
clap = "4"        # musicc CLI
dirs = "5"        # 路径处理
rand = "0.8"      # shuffle 随机
```

## 9. 开发说明

### 9.1 构建
```bash
cargo build
```

### 9.2 运行
```bash
# 终端 1: 启动 daemon
cargo run --package musicd

# 终端 2: 使用 CLI
cargo run --package musicc status
```

### 9.3 注意事项
- musicd 需先启动，musicc 才能连接
- 使用绝对路径添加文件可避免工作目录问题
- macOS 配置目录为 `~/Library/Application Support/music`