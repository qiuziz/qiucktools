# QuickTools

QuickTools 是一个基于 Tauri 2 + React 的桌面工具面板，用来把常用命令、脚本、打开操作和通知动作配置成可点击的小工具。应用提供工具列表、参数输入、执行结果展示、历史日志和基础设置，适合放在 macOS 菜单栏中快速执行本地自动化任务。

## 功能

- 菜单栏托盘入口，支持从托盘打开/隐藏主窗口
- 从 `tools.json` 加载工具配置
- 支持 `shell`、`script`、`open`、`notification` 四类工具
- 支持工具参数占位符，例如 `{{sourceP7b}}`
- 执行结果展示，包括状态、耗时、退出码、标准输出和错误输出
- SQLite 记录执行历史，支持分页和状态筛选
- 系统通知提示执行结果
- 中 / 英 / 日三语言界面
- 浅色、深色、跟随系统主题

## 技术栈

- 前端：React 18、TypeScript、Vite 7
- 桌面端：Tauri 2、Rust
- UI：Tailwind CSS、shadcn/ui、Radix UI、lucide-react
- 状态与数据：Tauri commands、SQLite、rusqlite
- 国际化：i18next、react-i18next
- 测试：Vitest、Cargo test

## 快速开始

### 环境要求

- Node.js 20
- pnpm
- Rust toolchain
- macOS 12 或更高版本

### 安装依赖

```bash
pnpm install
```

### 开发运行

```bash
pnpm tauri dev
```

开发模式会启动 Vite 渲染进程，并打开 Tauri 桌面应用。

### 构建

```bash
pnpm tauri build
```

macOS 构建产物位于：

```text
src-tauri/target/release/bundle/macos/QuickTools.app
```

## 工具配置

应用启动时会从仓库根目录的 `tools.json` 加载工具配置：

```text
~/work/quicktools/tools.json
```

配置示例：

```json
[
  {
    "id": "git-status",
    "name": "Git Status",
    "icon": "terminal",
    "description": "查看当前分支状态",
    "type": "shell",
    "command": "git status",
    "workingDir": "~",
    "params": [],
    "enabled": true,
    "timeoutMs": 60000,
    "sortOrder": 0
  }
]
```

字段说明：

| 字段 | 说明 |
| --- | --- |
| `id` | 工具唯一 ID |
| `name` | 工具名称 |
| `icon` | 工具图标，目前常用值包括 `terminal`、`file-code`、`folder-open`、`bell` |
| `description` | 工具描述 |
| `type` | 工具类型：`shell`、`script`、`open`、`notification` |
| `command` | 要执行的命令、脚本路径、打开目标或通知文本 |
| `workingDir` | 执行目录，支持 `~` |
| `params` | 参数定义列表 |
| `enabled` | 是否在工具列表中显示 |
| `timeoutMs` | 执行超时时间，单位毫秒 |
| `sortOrder` | 排序字段 |

### 参数占位符

工具命令可使用 `{{参数名}}` 占位符。执行时，用户输入会被安全地 shell quote 后替换到命令中。

```json
{
  "id": "replace-p7b",
  "name": "Replace P7B",
  "type": "shell",
  "command": "bash scripts/replace-p7b.sh {{sourceP7b}} {{branch}}",
  "workingDir": "~/work/quicktools",
  "params": [
    {
      "name": "sourceP7b",
      "label": "源 p7b 文件路径",
      "type": "text",
      "required": true
    },
    {
      "name": "branch",
      "label": "目标分支",
      "type": "text",
      "required": false,
      "default": ""
    }
  ],
  "enabled": true,
  "timeoutMs": 300000,
  "sortOrder": 0
}
```

## 工具类型

### shell

使用 `sh -c` 执行 `command`。

```json
{
  "type": "shell",
  "command": "git status",
  "workingDir": "~/work/quicktools"
}
```

### script

按文件扩展名选择解释器：

- `.sh` 使用 `bash`
- `.js` 使用 `node`

```json
{
  "type": "script",
  "command": "~/work/quicktools/scripts/example.sh"
}
```

### open

macOS 下使用系统 `open` 命令打开目标。

```json
{
  "type": "open",
  "command": "code ~/work/brain"
}
```

### notification

不执行外部进程，直接产生一次成功结果并触发系统通知。

```json
{
  "type": "notification",
  "command": "打开日志页面"
}
```

## 数据与日志

应用数据目录：

```text
~/Library/Application Support/QuickTools
```

主要文件：

- `quicktools.db`：SQLite 数据库，保存工具快照、设置和执行日志
- `logs/quicktools.log`：应用运行日志
- `crash.log`：崩溃日志

执行日志会记录工具 ID、工具名、参数、执行状态、耗时、退出码、标准输出、错误输出和错误信息。

## 常用命令

```bash
# 前端类型检查
pnpm typecheck

# 前端单元测试
pnpm test:unit

# Rust 测试
cd src-tauri && cargo test

# 格式化前端代码
pnpm format

# 检查前端格式
pnpm format:check
```

## 项目结构

```text
.
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   ├── hooks/              # 数据加载与执行 hooks
│   ├── i18n/               # 语言资源
│   ├── lib/api/            # Tauri command 调用封装
│   └── types/              # TypeScript 类型
├── src-tauri/              # Tauri / Rust 后端
│   ├── src/commands/       # Tauri commands
│   ├── src/database/       # SQLite schema、DAO、迁移
│   ├── src/services/       # 工具执行与通知
│   └── src/tray.rs         # 托盘菜单
├── scripts/                # 本地脚本
├── tools.json              # 工具配置
└── package.json
```

## 注意事项

- 当前 README 描述的是本仓库的 QuickTools 应用，不是上游 CC Switch 项目。
- `tools.json` 中的命令会在本机执行，新增工具前应确认命令来源可信。
- `open` 类型目前只在 macOS 上可用。
- 执行输出会被截断保存，单字段最多保留约 2000 个字符。
