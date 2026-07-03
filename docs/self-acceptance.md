# 自验收报告：对照 target-effect.md

> 逐条核对 [target-effect.md](./target-effect.md) 的目标效果与实际交付。
> 标注：✅ 完成 · 🟡 部分/降级 · ⏳ Windows 活体待验 · 📋 follow-up。
> **验收边界**：开发机 macOS 无 Rust 运行时的 LoL 客户端/游戏，只能验编译+单测+前端 build；LCU 连接、浮层覆盖、出装写入、Blitz 国服可达性等**活体行为须 Windows 验**（用户已知：实际运行在 Windows）。

## ✅ 前置 gate 已过（2026-07-04 SSH 实测）

- **Blitz 国服可达性（深审 F1）**：从用户家中 Windows（国服网络、无代理）SSH 实测 `datalake.v2.iesdev.com` → **HTTP 200 / 1.4s / 返回真实数据**。可实时取数；出厂快照仍作断网兜底。cdragon 同样可达。

## ① 游戏内浮层（北极星）

| 目标 | 状态 | 证据 / 说明 |
|---|---|---|
| 常驻 OS 置顶透明窗 | ✅编译 ⏳覆盖 | tauri.conf overlay 窗(transparent/alwaysOnTop/skipTaskbar) + overlay.rs；无边框覆盖行为 Windows 验 |
| 自动识别本局英雄 | ✅单测 ⏳活体 | U4 `get_current_champion_id`(cell→championId)，4 单测含 reroll；活体 LCU 轮询 Windows 验 |
| 海克斯优先级排序+top三连+核心装 | ✅ | U3 排序单测 + U8 MayhemBuild（稀有度分组×胜率降序），前端 build 通过 |
| 默认点击穿透（F4） | ✅编译 ⏳ | `set_ignore_cursor_events(true)`；穿透/焦点三态 Windows 验 |
| 热键切换 | 🟡 | toggle 命令 ✅；**全局热键因 macOS 与 tauri2.7 依赖冲突延到 Windows 侧接入** |
| 不替你选三选一 | ✅ | 设计边界，只给排序对照（官方 API 无三选一数据） |

## ② 客户端选英雄出装助手

| 目标 | 状态 | 说明 |
|---|---|---|
| 展示海克斯大乱斗出装/海克斯 | ✅ | MayhemCodexView + MayhemBuild，build 通过 |
| 一键导入预设出装 | ✅单测 ⏳活体 | U5 `apply_mayhem_item_set`，**schema 按深审 F2 修正**(wrapper+associatedMaps:[12]嚎哭深渊)；build+merge 单测✓（保用户 set）；真写入 Windows 验 |
| 一键导入符文 | ✅ | loop tick1 接入：复用既有 op.gg 符文路径，「导入符文」按钮；build 验证 |
| 停靠客户端右侧独立窗 | 🟡 | v1 做主窗内视图（计划已定 deferred 独立停靠窗） |

## ③ 速查库

| 目标 | 状态 | 说明 |
|---|---|---|
| 搜任意英雄看推荐 | ✅ | GUI 重做：ChampionSelector 搜索+173 英雄头像网格（数字 id 输入已废除） |
| 全局海克斯 tier 表 | ✅ | loop tick2：快照 173 英雄本地加权聚合（≥500 局），tab「全局海克斯榜」；97 测试+GUI 截图验证 |

## 贯穿项

| 目标 | 状态 | 说明 |
|---|---|---|
| 智能=数据驱动（非静态标签） | ✅ | 排序只依赖 win_rate，单测断言；不做 AP/AD 标签 |
| KR 代理数据 + UI 标来源/版本 | ✅ | MayhemBuild 显示「数据来源 KR · 版本 X」 |
| 出厂快照兜底（F1/F5） | ✅单测 | U10 三层取数(实时→快照)，snapshot 单测✓ |
| 平台 Windows | ✅ | GitHub Actions windows-latest 云打包+签名发布（v0.1.2+ 自动更新）；F1 Blitz 国服直连已 SSH 实测通过 |

## 红线（明确不做）—— 全部守住

- ✅ 不注入内存 / 不读屏 / 不 hook DirectX（纯 LCU + OS 置顶窗）
- ✅ 不自动识别替选游戏内三选一
- ✅ 不伪造国服精准数据（外服代理如实标 KR）

## Mac 侧已验证清单

- Rust：`cargo test --lib` **96 passed / 0 failed**（blitz 解码 / augments 覆盖 / mayhem 排序 / item_set 构建合并 / champ_select 提取 / snapshot / 既有 ts-rs 导出）
- Rust：`cargo build --lib` 编译通过（含 overlay 窗 + 快照载入）
- 前端：`vite build` 通过（我的 4 个文件类型干净；基线 GameTypeSelector 既有类型错误非本次引入，未动）
- 数据源：Blitz 端点 + cherry-augments 覆盖 + 图标 URL 变换均实测确认

## 待 Windows 验收清单（交给用户真机）

1. ~~Blitz 国服可达性~~ ✅ 已 SSH 实测通过
2. LCU 连接 + 选英雄自动识别当前英雄
3. 出装真写入客户端、局内商店按模式可见（验 map=12）
4. 浮层无边框覆盖 + 点击穿透 + 不抢焦点 + DPI
5. 全局热键接入（tauri-plugin-global-shortcut，Windows 无 macOS 依赖冲突）
6. 出厂快照在断网/不可达时兜底生效
