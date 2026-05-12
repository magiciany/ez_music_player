# EZ Music Player

基于 Unix Socket 通信的本地音乐播放器，采用 C/S 架构。

## 功能特性

- 🎵 支持 MP3、M4A、WAV 等常见音频格式
- 📋 播放队列管理（添加、移除、移动、清空）
- 🔀 随机播放和循环模式（关闭、单曲循环、全部循环）
- 🔌 Unix Domain Socket 通信，本地快速响应
- 📁 自动保存播放队列

## 项目结构

```
ez_music_player/
├── music-common/    # 共享类型和工具
├── musicc/          # CLI 客户端
├── musicd/          # 后台守护进程
├── Cargo.toml       # Workspace 配置
└── LICENSE          # MIT License
```

## 快速开始

### 构建

```bash
cargo build
```

### 启动服务

```bash
# 在一个终端启动后台服务
cargo run --package musicd
```

### 使用 CLI

```bash
# 添加音乐文件
cargo run --package musicc -- add /path/to/your/music.mp3

# 查看队列
cargo run --package musicc -- list

# 播放
cargo run --package musicc -- play

# 其他命令
cargo run --package musicc -- pause      # 暂停
cargo run --package musicc -- stop      # 停止
cargo run --package musicc -- next      # 下一首
cargo run --package musicc -- prev      # 上一首
cargo run --package musicc -- shuffle   # 随机打乱
cargo run --package musicc -- repeat off # 设置循环模式
cargo run --package musicc -- status    # 查看状态
cargo run --package musicc -- quit      # 关闭服务
```

## 配置目录

数据存储在：`~/Library/Application Support/ez_music_player/`

| 文件 | 说明 |
|------|------|
| `musicd.sock` | Unix Domain Socket |
| `musicd.pid` | 进程 ID |
| `queue.json` | 播放队列 |

## License

MIT License - see [LICENSE](LICENSE) 文件