# samsara 命令参考

> 返回：[README.md](../README.md) · [English](commands.en.md)

---

## 初始化

| 命令 | 说明 |
|------|------|
| `samsara init [--yes]` | 初始化知识库，注入自我进化协议到 AGENTS.md |

## 教训管理

| 命令 | 说明 |
|------|------|
| `samsara write <domain> <keyword>` | occurrences +1（已存在时） |
| `samsara write <domain> <keyword> --summary "..."` | 写入 / 更新教训内容 |
| `samsara write <domain> <keyword> --type error\|skill\|pattern\|insight` | 指定类型 |
| `samsara write <domain> <keyword> --verify` | 标记规则已验证有效 |
| `samsara search <query>` | 全文搜索知识库 |
| `samsara search <query> --domain <d>` | 限定 domain 搜索 |
| `samsara search <query> --type <t>` | 限定类型搜索 |
| `samsara archive <domain> <keyword>` | 归档教训（不再活跃） |

## 晋升 / 降级

| 命令 | 说明 |
|------|------|
| `samsara promote <domain> <keyword>` | 晋升为规则（写入 `rules/<domain>.md`） |
| `samsara promote <domain> <keyword> --layer0` | 晋升并写入 `AGENTS.md`（每次启动生效） |
| `samsara demote <pattern>` | 从 AGENTS.md 移除规则 |
| `samsara demote <pattern> --yes` | 跳过确认直接移除 |
| `samsara prime` | 推荐晋升候选（occurrences 最高的未晋升教训） |
| `samsara prime --limit <N>` | 指定返回数量 |
| `samsara prime --domain <d>` | 限定 domain |

## 分析与维护

| 命令 | 说明 |
|------|------|
| `samsara status` | 知识库统计概览 |
| `samsara lint` | 检查知识库健康度（格式、重复等） |
| `samsara lint --fix` | 自动修复可修复的问题 |
| `samsara reflect` | 分析学习模式（哪些 domain 最活跃等） |
| `samsara log` | 查看操作日志 |
| `samsara log --tail <N>` | 最近 N 条记录 |
| `samsara log --action <t>` | 按操作类型过滤 |
| `samsara log --rotate` | 轮转日志文件 |
| `samsara skill-note <name>` | 记录 skill 使用情况 |
| `samsara skill-note <name> --fail` | 标记 skill 执行失败 |
| `samsara skill-note <name> --note "..."` | 附加备注 |

## Domain 管理

| 命令 | 说明 |
|------|------|
| `samsara domain list` | 列出所有 domain |
| `samsara domain add <name>` | 注册新 domain |

## 同步

| 命令 | 说明 |
|------|------|
| `samsara remote add <name> <url>` | 添加同步远端 |
| `samsara remote set <name> <url>` | 修改远端地址 |
| `samsara remote show` | 查看当前远端配置 |
| `samsara push` | 推送到远端 |
| `samsara push --dry-run` | 模拟推送（不实际执行） |
| `samsara pull` | 从远端拉取 |

## 其他

| 命令 | 说明 |
|------|------|
| `samsara self-update` | 升级到最新版本 |
| `samsara self-update --check` | 仅检查是否有新版本 |
| `samsara mcp serve` | 启动 MCP 服务（由 AI 工具自动调用，无需手动执行） |
