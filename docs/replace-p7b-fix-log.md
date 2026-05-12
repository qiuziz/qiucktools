# Replace P7B 工具修复记录

## 问题现象

QuickTools 的 Replace P7B 工具执行失败，报错 `错误: 源文件不存在: '/Users/qiuz/Downloads/app/ajk-harmony-debugDebug(4).p7b'`。

注意路径中包含了多余的单引号字符 `'`，实际文件在 `/Users/qiuz/Downloads/app/ajk-harmony-debugDebug(4).p7b`。

## 根本原因

### 第一层：Rust `shell_quote` 对空值返回 `''`

`shell_quote("")` 返回 `''`，即两个单引号字符。当 branch 参数为空时：
```rust
// 原代码
fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")  // 空值 -> "''"
}
```

生成的命令变成：
```bash
bash scripts/replace-p7b.sh '/Users/...' '' ''
```

此时 `''` 被当成了一个字面量空字符串参数，而非空值。

### 第二层：`sh -c` 的嵌套引号问题

当路径中包含 `'` 时（如 `ajk-harmony-debugDebug(4).p7b`），`shell_quote` 生成：
```
'/Users/qiuz/Downloads/app/ajk-harmony-debugDebug(4).p7b'
```

但实际的 `sh -c "..."` 调用中，如果整个 `-c` 字符串本身用双引号包裹，`''` 会被解析为字面量单引号，导致路径变形。

生成的完整命令（`/tmp/quicktools_cmd.txt` 实际内容）：
```
bash scripts/replace-p7b.sh ''"'"'/Users/qiuz/Downloads/app/ajk-harmony-debugDebug(4).p7b'"'"''
```

这就是为什么日志中 `{{sourceP7b}}` 消失了——被引号嵌套完全吞掉了。

### 第三层：前端粘贴带引号

用户可能从命令行输出中复制路径粘贴到输入框，路径已经被单引号包裹过。这是最直接的触发原因。

## 修复措施

### 1. 空值不转引号（Rust 层）
```rust
fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return String::new();  // 空值返回空字符串，不生成 "''"
    }
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}
```

同时 `substitute_params` 对空值跳过替换：
```rust
if value.is_empty() {
    acc.replace(&placeholder, "")  // 空值直接删除占位符
} else {
    // 正常处理
}
```

### 2. 前端输入清理（防御层）
在 `ParamDialog.tsx` 中对输入值做清理，移除首尾配对出现的单引号：

```typescript
const cleanValue = (v: string) => {
  const trimmed = v.trim();
  if (trimmed.startsWith("'") && trimmed.endsWith("'") && trimmed.length >= 2) {
    return trimmed.slice(1, -1);
  }
  return v;
};
```

### 3. 脚本支持 Key-Value 模式（备用方案）
修改 `replace-p7b.sh` 支持两种参数模式：

```bash
# 传统模式
./replace-p7b.sh <path> [branch]

# Key-value 模式（由 QuickTools executor 调用）
./replace-p7b.sh sourceP7b '/path/to/file.p7b' branch ''
```

## 经验教训

1. **空字符串和字面量引号 `''` 不同**。空字符串用 `[[ -z "$var" ]]` 检测，但 `[[ "$var" == "''" ]]` 只匹配字面量两个单引号。Rust 生成的 `''` 是 shell 中的字面量空字符串，会被当成参数值而非空值。

2. **`sh -c` 的引号嵌套陷阱**。当通过 `Command::new("sh").args(&["-c", "full command string"])` 执行时，整个命令字符串已经用双引号包裹。如果字符串内容包含 `'`，Rust 的 `shell_quote` 生成的引号序列在双引号上下文中会被不同地解析，导致引号匹配崩溃。

3. **前端输入要防御性处理**。用户输入可能来自命令行输出（已带引号）、复制粘贴、缓存数据等。应该在数据进入系统前做清理，而非依赖后端修复。

4. **调试信息要写到位**。这次通过 `/tmp/quicktools_cmd.txt` 直接看到 Rust 生成的实际命令字符串，是定位问题的关键。之前只看数据库日志不够，要看原始数据。

## 相关文件

- `src-tauri/src/services/executor.rs` - `shell_quote` 和 `substitute_params` 修复
- `src/components/ParamDialog.tsx` - 前端输入清理
- `scripts/replace-p7b.sh` - Key-value 模式支持
