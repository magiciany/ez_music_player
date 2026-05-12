# Music Player 项目总结

## 1. 问题修复

### 1.1 musicc 命令卡住无响应
**根因**：Socket 通信协议使用 `read_to_end()` 读取响应，当 musicd 关闭连接时，macOS 上 `read_to_end()` 无法正确检测 EOF，一直等待。

**解决方案**：改用 `BufReader::read_line()` 基于行的读取协议。

**修改文件**：
- `music-common/src/socket.rs` — client 端改用 line-based read
- `musicd/src/main.rs` — server 端改用 line-based read

### 1.2 添加文件后播放无声音
**根因**：用户添加文件时传入相对路径（如 `audio_cache/file.mp3`），musicd 工作目录不确定，导致找不到文件。

**解决方案**：musicc 在发送 `Add` 命令前将相对路径转为绝对路径。

**修改文件**：
- `musicc/src/main.rs` — `to_command()` 函数中 `Add` 分支调用 `std::fs::canonicalize()`

## 2. 命名规范

### 2.1 避免与 Apple Music 冲突
| 旧名称 | 新名称 | 路径 |
|--------|--------|------|
| socket | musicd.sock | ~/Library/Application Support/music/musicd.sock |
| daemon.pid | musicd.pid | ~/Library/Application Support/music/musicd.pid |

**修改文件**：
- `musicd/src/main.rs`
- `musicc/src/main.rs`

### 2.2 macOS 配置目录
使用 `dirs::config_dir()` 在 macOS 上返回 `~/Library/Application Support/music`，而不是 `~/.config/music`。

## 3. 用户体验改进

### 3.1 无参数时显示帮助
`musicc` 不带参数时显示完整的帮助信息（而不是仅一行 usage）。

**修改文件**：
- `musicc/src/main.rs` — 使用 `Cli::command().render_help()`

## 4. Git 仓库规范

### 4.1 .gitignore 规则
```
/target       # 编译产物
Cargo.lock   # Cargo 锁文件
**/._*       # macOS 元数据文件
```

### 4.2 已清理的垃圾文件
- 所有 `._*` macOS 元数据文件
- `target/` 目录
- `Cargo.lock`

## 5. 当前目录结构

```
music/
├── Cargo.toml          # workspace 配置
├── .gitignore
├── music-common/       # 共享类型定义
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs     # Command, Response, Queue, PlaybackState 等类型
│       └── socket.rs  # SocketClient
├── musicc/             # CLI 客户端
│   ├── Cargo.toml
│   └── src/
│       └── main.rs    # 命令行入口
└── musicd/             # 后台服务
    ├── Cargo.toml
    └── src/
        ├── main.rs    # socket 监听和命令处理
        ├── handlers.rs # 命令处理器
        ├── audio.rs   # rodio 音频播放
        └── state.rs   # PlayerState, Queue 管理
```

## 6. 使用方法

```bash
# 启动后台服务
./target/debug/musicd

# CLI 命令
./target/debug/musicc add <path>     # 添加音乐文件（自动转绝对路径）
./target/debug/musicc play           # 播放
./target/debug/musicc pause          # 暂停
./target/debug/musicc stop           # 停止
./target/debug/musicc next           # 下一首
./target/debug/musicc prev           # 上一首
./target/debug/musicc list           # 查看播放队列
./target/debug/musicc shuffle        # 随机打乱队列
./target/debug/musicc repeat <mode>  # 设置循环模式 (off/one/all)
./target/debug/musicc status         # 查看播放状态
./target/debug/musicc quit           # 关闭后台服务

# 无参数时显示帮助
./target/debug/musicc
```