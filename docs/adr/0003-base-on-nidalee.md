# 基座续用 Nidalee（Tauri），不用 lolpro / 不新起

在 Nidalee（Tauri 2 + Vue3 + Rust）基础上开发，而非另一个高度相似的开源工具 lolpro（Electron + React，已做 ARAM Mayhem + Arena + HUD 浮层 + Blitz 源）或从零起。

## Considered Options

- **Nidalee（选中）**：Tauri（比 Electron 轻）、已有 LCU 连接/鉴权、符文导入、对局分析、自动接受/秒选。缺浮层、缺出装(item-set)写入、缺海克斯大乱斗源。
- **lolpro**：已做浮层+海克斯大乱斗+Blitz 源，但**仓库无 LICENSE**（默认保留全部权利，不可照搬代码），且**不导入 item-set/符文进客户端**——正是本项目要的差异化。
- 从零起：放弃 Nidalee 现成的 LCU + 对局分析地基，不划算。

## Consequences

- lolpro 的 Blitz Datalake 端点/查询是独立实测验证的**事实**（非其代码），可自研 Rust 查询层复现，不构成侵权。
- 差异化优势：Nidalee 已有对局分析 + LCU 写入地基，本项目在其上补出装写入 + 海克斯大乱斗浮层。
