---
title: "feat: 海克斯大乱斗助手（ARAM Mayhem Helper）"
type: feat
status: active
date: 2026-07-03
origin: docs/target-effect.md
depth: standard
---

# feat: 海克斯大乱斗助手（ARAM Mayhem Helper）

## Summary

在 Nidalee（Tauri 2 + Vue3 + Rust）基座上，为国服海克斯大乱斗玩家做三件事：**游戏内浮层看本局英雄的海克斯优先级**（北极星）、**选英雄时客户端一键导入出装**、**离线速查库**。数据主源 Blitz Datalake（外服 KR，实测直连），智能=数据驱动（真实胜率），不做游戏内三选一自动识别、不注入不读屏。

目标效果见 [docs/target-effect.md](../target-effect.md)；关键决策见 [docs/adr](../adr)（数据源、浮层实现、基座选型）。

分阶段交付，**Phase A+B 为最小闭环**（拉数据 + 识别当前英雄 → 打印推荐），跑通对味后再堆 UI。

---

## Problem Frame

海克斯大乱斗每局随机英雄、局内三档海克斯三选一（等级 7/11/15），出装被海克斯打乱、无定式。玩家需要「这局这英雄该优先抢哪些海克斯、走哪套三连、出什么装」的即时、真实战绩指导。官方 API 不暴露增强数据，国服无开放统计源——现实数据只能取外服(KR)作代理。

Nidalee 已有 LCU 连接/鉴权、符文导入、对局分析、gameflow/champ-select 轮询，但缺：海克斯大乱斗数据源、出装(item-set)写入、浮层窗口。本计划补齐这三块并组装成三个交付物。

---

## Requirements Trace

源自 [target-effect.md](../target-effect.md) 的三交付物 + 约束：

| 需求 | 交付物 | 实现单元 |
|---|---|---|
| R1 游戏内浮层看海克斯优先级（北极星） | ① | U7, U8（依赖 U1-U4） |
| R2 选英雄时一键导入出装+符文 | ② | U5, U6 |
| R3 离线速查库 | ③ | U9 |
| R4 对局感知=绑本局英雄，数据驱动 | 贯穿 | U4（识别）+ U3（排序） |
| R5 外服 KR 代理数据，UI 标来源/版本 | 贯穿 | U3（数据）+ U6/U8/U9（标注） |
| R6 平台 国服/Windows | 约束 | 见 Risks & Dependencies |
| 可用性兜底（冷启动/不可达） | 贯穿 | U10 出厂快照 |

---

## Key Technical Decisions

- **KTD1 数据源 = Blitz Datalake 外服代理**（见 [ADR-0001](../adr/0001-data-source-blitz-kr-proxy.md)）。`datalake.v2.iesdev.com/graphql`，`prod_aram_mayhem_champion` 查询按 champion_id 返回 augments/items/augment_trios（含胜率/选取率/tier/rank）。实测免鉴权直连、返回 Databricks 列式 payload。
- **KTD2 浮层 = OS 置顶窗，非注入**（见 [ADR-0002](../adr/0002-overlay-topmost-window-not-injection.md)）。Tauri 第二窗口 transparent + alwaysOnTop + decorations:false，游戏须无边框模式。热键呼出/收起用 Tauri global-shortcut。
- **KTD3 基座 = 续用 Nidalee**（见 [ADR-0003](../adr/0003-base-on-nidalee.md)）。复用 LCU/对局分析/符文导入；Blitz 查询层自研 Rust（lolpro 无 License，只借鉴实测得到的端点事实，不搬代码）。
- **KTD4 海克斯 id→名称/图标**（源已实测确认）：Blitz 只给 augment id（如 2095/1401）。映射源 = CommunityDragon **`cherry-augments.json`**（非 `arena/en_us.json`——后者是斗魂竞技场海克斯、id 空间不同、不含 Mayhem id）。该文件 638 条中 **170 条 `augmentNameId` 以 `ARAM_` 开头 = 海克斯大乱斗海克斯**，id 与 Blitz 对齐。字段：`id` / `nameTRA`(名) / `augmentSmallIconPath` / `rarity`(`kSilver`/`kGold`/`kPrismatic`→白银/黄金/棱彩)。中文名走 `.../global/zh_cn/v1/cherry-augments.json`（id 2095="掷骰狂人"）。图标 URL 变换：小写 `augmentSmallIconPath`，`/lol-game-data/`→`https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/`，并塌缩 `/assets/assets/`→`/assets/`（实测 200）。item id 为标准 Riot 物品 id，走 Nidalee 现有 `ddragon`。
- **KTD5 出装写入 schema**（审查 F2 修正）：`PUT /lol-item-sets/v1/item-sets/{summonerId}/sets`，body 是 wrapper `{accountId, timestamp, itemSets:[{title, associatedChampions:[championId], associatedMaps:[12], blocks}]}`。⚠️ **不复用** `types.rs::ItemSet`（那是 op.gg 形状、单 `champion:String`/`mode:CHERRY`——CHERRY 是斗魂竞技场、Mayhem 在嚎哭深渊 map 12）；U5 新建专用写入类型，实现首步 GET 真实 sets 抓 fixture 校准。Nidalee **无既有 item-set 写入**（perks 只写符文），U5 纯新建。
- **KTD6 命令注册**：新 Tauri command 统一在 `src-tauri/src/lib.rs` 的 `generate_handler!` 注册，沿仓库 `lcu::<mod>::commands::<fn>` 惯例。
- **KTD7 Blitz 可达性（未验证·前置 gate，审查 F1）**：本会话所有「实测直连」均经开发机 Surge 代理/TUN（连到伪 IP `198.18.x.x`），**未证实国服无代理网络能达美国 `datalake.v2.iesdev.com`**。这是最 load-bearing 的未验证假设——须在 Windows 目标 + 家庭网络实测，作为依赖 Blitz 的前置 gate。配 U10 出厂快照兜底，使不可达时仍可降级运行。

---

## Output Structure

新增模块（相对 mayhem-helper 仓库根）：

```
src-tauri/src/
├── data/                      # 新：外部数据源层
│   ├── blitz/                 # U1  Blitz Datalake 查询 + 解码
│   │   ├── mod.rs
│   │   ├── query.rs
│   │   └── decode.rs
│   └── augments/              # U2  海克斯元数据 (id→name/icon/rarity)
│       ├── mod.rs
│       └── metadata.rs
├── common/commands/
│   └── mayhem.rs              # U3  海克斯大乱斗数据模型 + 命令
└── lcu/item_sets/            # U5  出装(item-set)写入
    ├── mod.rs
    ├── service.rs
    └── commands.rs

src/
├── composables/mayhem/
│   └── useMayhemData.ts       # U6  取数 composable（U6/U8/U9 共用）
├── components/features/mayhem/
│   └── BuildPanel.vue         # U6  选英雄出装助手面板
└── views/
    ├── OverlayView.vue        # U8  浮层决策卡
    └── MayhemCodexView.vue    # U9  速查库
```

浮层第二窗口在 `src-tauri/tauri.conf.json` 声明（U7）。以上为预期形状，实现时可微调；各单元 **Files** 为准。

---

## Implementation Units

> 分五阶段，依赖顺序排列。Phase A+B（U1-U4）= 最小闭环。

### U1. Blitz Datalake 查询层

**Goal**: 能对任意 champion_id 拉到海克斯大乱斗原始数据行。
**Requirements**: R1, R4, R5（数据地基）
**Dependencies**: 无
**Files**:
- `src-tauri/src/data/blitz/mod.rs`（新）
- `src-tauri/src/data/blitz/query.rs`（新，GraphQL 查询常量 + 端点）
- `src-tauri/src/data/blitz/decode.rs`（新，Databricks 列式 payload 解码）
- `src-tauri/src/data/blitz/decode.rs` 内 `#[cfg(test)]` 或 `src-tauri/src/data/blitz/tests.rs`
- `src-tauri/src/lib.rs`（挂 `mod data;`）
**Approach**: POST `datalake.v2.iesdev.com/graphql`，query `prod_aram_mayhem_champion`（param champion_id）。响应 `data.executeDatabricksQuery.payload` 含 `manifest.schema.columns` + `result.dataArray`；按 column position + typeName 解码成 `Vec<HashMap<String,Value>>`，STRUCT/MAP/ARRAY 递归 JSON parse。复用 Nidalee 现有 reqwest client 与错误处理惯例（参考 `src-tauri/src/common/commands/builds.rs` 的 op.gg 请求）。
**Patterns to follow**: `src-tauri/src/lcu/opgg/client.rs`（外部 HTTP 客户端）、`builds.rs::get_champion_build_new`（reqwest + UA + json 解析）。
**Test scenarios**:
- Happy: 用真实响应 fixture 解码 `prod_aram_mayhem_champion`（champion_id=5），断言得到 1 行、列含 `champion_id/data/patch/dt`，`data` 递归解析出 `augments`/`items`/`augment_trios` 三个 dict。
- Edge: 空 `dataArray` → 返回空 Vec 不 panic。
- Error: HTTP 非 200 → 返回 Err 带状态码；payload 缺 `manifest` → fail-loud 报错，不静默返回空。
**Verification**: 单测通过；临时 bin/命令对 champion_id=5 打印出 augments 数量 >0。

### U2. 海克斯元数据（id→名称/图标/稀有度）

**Goal**: 把 Blitz 的 augment id / item id 映射成可展示的名称、图标 URL、稀有度。
**Requirements**: R1, R3（展示地基）
**Dependencies**: 无（可与 U1 并行）
**Files**:
- `src-tauri/src/data/augments/mod.rs`（新）
- `src-tauri/src/data/augments/metadata.rs`（新，拉取/缓存 CommunityDragon **cherry-augments**）
- `src-tauri/src/data/augments/fixtures/cherry-augments-zh_cn.json`（新，测试 fixture）
- `src-tauri/src/data/augments/metadata.rs` 内测试
**Approach**（源已实测确认，见 KTD4）: 取 CommunityDragon **`cherry-augments.json`**（**全集查表，非只 filter `ARAM_` 子集**——实测 Blitz 164 个 augment id 100% 命中全集、仅 70% 命中 ARAM_ 子集，部分 Mayhem 复用基础 cherry augment）。zh_cn locale 取中文名。构建 `HashMap<u32, AugmentMeta{name_zh, icon_url, rarity}>`。稀有度映射 `kSilver/kGold/kPrismatic`→白银/黄金/棱彩。图标 URL 变换见 KTD4（塌缩 `/assets/assets/`→`/assets/`，已验 200）。item id→名称/图标复用 Nidalee 现有 ddragon（`src-tauri/src/lcu/ddragon.rs`）。
**Patterns to follow**: `src-tauri/src/lcu/ddragon.rs`（静态资源拉取+缓存）。
**Test scenarios**:
- Happy: fixture，`resolve(2095)` 返回 name="掷骰狂人" + icon URL + rarity=棱彩。
- 覆盖率（Covers F3）: 断言某英雄 Blitz augment id 集合对 cherry-augments 全集命中率 100%（缺失即 fail，防静默错位映射）。
- Edge: 未知 id → 返回占位（id 字符串 + 默认图标），不报错。
**Verification**: 单测通过；Blitz id 集合全命中 cherry 全集；已知 id 解析出正确中文名。

### U3. 海克斯大乱斗数据模型 + Tauri 命令

**Goal**: 暴露 `get_mayhem_champion(champion_id)`，返回排好序的海克斯优先级、核心出装、top 三连组合。
**Requirements**: R1, R3, R4, R5
**Dependencies**: U1, U2
**Files**:
- `src-tauri/src/common/commands/mayhem.rs`（新，命令 + 数据整形）
- `src-tauri/src/common/commands/mod.rs`（挂模块）
- `src-tauri/src/lib.rs`（`generate_handler!` 注册命令）
- `src/types/generated/`（ts-rs 导出类型，跟随仓库 `pnpm types` 流程）
- `src-tauri/src/common/commands/mayhem.rs` 内测试
**Approach**: 调 U1 取行 → 解 `data` blob → 整形为：`augments` 按 win_rate 降序、按稀有度分组，附 U2 元数据；`items` 按 win_rate/pick_rate 取核心；`augment_trios` 解析 `"1020:1138:2102"` key 按胜率排序取 top N。附 patch + 来源标识（KR）随数据返回，供 UI 标注（R5）。**排序是纯函数，提取为模块级函数便于测**。
**Patterns to follow**: `src-tauri/src/common/commands/builds.rs`（命令签名 + `#[tauri::command]`）；ts-rs 类型导出见 `src-tauri/src/lcu/types.rs`。
**Test scenarios**:
- Happy: fixture → augments 按 win_rate 降序、分组正确；trio key `"a:b:c"` 解析为 3 个 id 且按胜率排序。
- Edge: 某英雄无 trio 数据 → 返回空 trio 列表不报错；win_rate 为字符串 "0.5" → 正确转 float 排序。
- Covers 数据驱动(R4): 断言排序只依赖 win_rate 数值，不依赖英雄静态标签。
**Verification**: `pnpm type-check` + 单测通过；前端 invoke 该命令拿到结构化推荐。

### U4. 当前英雄识别（最小闭环收口）

**Goal**: 从 LCU 识别玩家本局英雄（选英雄阶段 + 游戏内），驱动推荐自动跟随。
**Requirements**: R4
**Dependencies**: U3
**Files**:
- `src-tauri/src/lcu/champ_select/service.rs`（扩展，取本地玩家 championId）
- `src-tauri/src/lcu/champ_select/commands.rs`（新命令或事件）
- `src-tauri/src/lib.rs`（注册）
- `src/composables/game/`（前端订阅当前英雄变化，复用现有轮询）
**Approach**（架构选择已定，见审查 F6/可行性#2）: **选英雄阶段**从 champ-select session 取本地玩家英雄——`local_player_cell_id` → `my_team[cell].champion_id`（`ChampSelectSession`/`ChampSelectPlayer` 字段已在 `types.rs`，`get_champ_select_session_typed` 已注册）。**把选定 championId 缓存进状态**。**进对局(gameflow InProgress)后直接复用该缓存驱动 U8**——ARAM 英雄开局锁定不变，**不依赖 Live Client Data(:2999)**（其 playerlist 只给 `championName` 字符串、不标本地玩家，反查成本高，v1 不走）。海克斯大乱斗 champ-select 是随机分配 + `benchChampions`(备选席) + reroll + trade、**无 ban/pick action**——用 cell→champion_id 读取，不走 actions。英雄变化发事件，前端调 U3。
**Patterns to follow**: `src-tauri/src/lcu/champ_select/service.rs`（`local_player_cell_id`+`my_team[].champion_id`）、`src-tauri/src/lcu/unified_polling.rs`（事件广播）。
**Test scenarios**:
- Happy: 真实**海克斯大乱斗** champ-select session fixture → `ChampSelectSession` 能反序列化，提取本地玩家 championId 正确。
- Integration（Covers F6，大乱斗主路径）: reroll/trade 后 `my_team[cell].champion_id` 变化 → 发出的当前英雄事件随之更新（非依赖 actions）。
- Edge: 未分配（championId=0/None）→ 返回 None，不误触发。
**Verification（含最小闭环）**: 客户端在海克斯大乱斗选英雄/局内时，程序自动打印「当前英雄的 top 海克斯 + top 三连 + 核心装」。**此即 Phase A+B 最小闭环验收点（需 Windows + 活体客户端 + 模式开放，见 Risks）——数据对味后再进 Phase C。** 离线部分（session 反序列化）用 fixture 在 Mac 先验。

### U5. LCU 出装（item-set）写入

**Goal**: 把当前英雄的海克斯大乱斗核心出装写成客户端预设出装，局内商店可见。
**Requirements**: R2
**Dependencies**: U3
**Files**:
- `src-tauri/src/lcu/item_sets/mod.rs`（新）
- `src-tauri/src/lcu/item_sets/types.rs`（新，**专用 LCU 写入 schema**，不复用 op.gg 形状的 `types.rs::ItemSet`）
- `src-tauri/src/lcu/item_sets/service.rs`（新，GET 合并 + PUT）
- `src-tauri/src/lcu/item_sets/commands.rs`（新）
- `src-tauri/src/lcu/mod.rs` + `src-tauri/src/lib.rs`（挂载 + 注册）
**Approach**（schema 已按审查 F2 + 可行性#3 修正，⚠️ 需真机 GET 校准）:
- **纠正既有认知**：`types.rs::ItemSet{title,champion:String,mode,map,blocks}` 是 **op.gg 出装解析形状，非 LCU 写入 schema**；Nidalee **无既有 item-set 写入路径**（`perks` 只写 `/lol-perks` 符文）。U5 是纯新建。
- LCU `PUT /lol-item-sets/v1/item-sets/{summonerId}/sets` 的 body 是**外层 wrapper**：`{ accountId, timestamp, itemSets: [ { title, associatedChampions:[int], associatedMaps:[int], blocks:[...] } ] }`。关键：用 **`associatedChampions:[championId]` + `associatedMaps:[12]`（嚎哭深渊/ARAM，非 CHERRY/Arena）**，不是单个 `champion:String`、不是 `mode:"CHERRY"`。
- **实现首步先 `GET` 一次真实 sets 抓 fixture 校准字段**（summonerId vs accountId 来源、wrapper 确切字段名）——现有 `summoner::service` 是否返回 accountId 要核。
- 先 GET 现有 sets → 合并/替换本工具 set（标题带固定前缀识别）→ 整体 PUT 回写，**不覆盖用户自定义 set**（持久状态：先读后写、不裸覆盖）。
**Patterns to follow**: `src-tauri/src/lcu/perks/service.rs`（LCU 写入 + 端点封装惯例）；`src-tauri/src/lcu/summoner/service.rs`（取 summoner/accountId）。
**Test scenarios**:
- Happy: 给定 items + championId + accountId，构建的 wrapper body 结构合法（含 `associatedChampions`/`associatedMaps:[12]`/`blocks`）。
- Edge: 已有同名工具 set → 替换而非重复追加；用户自定义 set 保留不丢。
- Error: accountId/summonerId 取不到（未登录）→ 明确错误，不写入。
**Verification（Windows 活体）**: 命令执行后，海克斯大乱斗**局内商店按模式过滤能看到**导入出装（验 map=12 正确）；重复导入不产生重复 set、不覆盖用户既有 set。Mac 侧只验 body 构建单测。

### U6. 出装助手停靠面板（Vue）

**Goal**: 选/切英雄时展示海克斯大乱斗出装+符文，提供一键导入按钮。
**Requirements**: R2, R5
**Dependencies**: U3, U4, U5（导入按钮依赖 U5；符文导入 Nidalee 已有）
**Files**:
- `src/composables/mayhem/useMayhemData.ts`（新，封装 invoke `get_mayhem_champion` + 缓存，U6/U8/U9 共用）
- `src/components/features/mayhem/BuildPanel.vue`（新）
- `src/views/` 或现有布局挂载入口 + `src/router`（如需）
**Approach**: 订阅 U4 当前英雄 → useMayhemData 取数 → 展示海克斯优先级/核心装/符文 + 两个按钮：「导入出装」（U5 命令）、「导入符文」（复用现有 `apply_champion_build` 的符文路径）。显眼标注「数据来源 KR / 版本 X」（R5）。停靠形态：优先复用主窗内视图；「贴客户端右侧」的独立窗口可与 U7 浮层窗机制共用，**实现时二选一先做主窗内视图**（更省，先落地）。
**Patterns to follow**: `src/components/features/`（现有 feature 组件）、shadcn-vue 组件、`@tanstack/vue-query`（若仓库取数用它）。
**Test scenarios**（vitest，若仓库已配）:
- Happy: 传入当前英雄 → 面板渲染出海克斯排序 + 核心装 + 来源/版本标签。
- Edge: 取数失败 → 显示错误态，不白屏。
- 开关两侧: 点「导入出装」触发 U5 命令且带正确 champion；不点则不调用。
- Test expectation: 若仓库无 vitest 配置，则以手动验收替代（见 Verification）。
**Verification**: 海克斯大乱斗选英雄时面板自动出对应英雄内容；点按钮出装/符文成功写入客户端。

### U7. 透明置顶浮层窗口

**Goal**: 游戏运行时可呼出的置顶半透明窗口容器 + 热键切换。
**Requirements**: R1（北极星容器）
**Dependencies**: 无（可与数据层并行，但 UI 内容依赖 U8）
**Files**:
- `src-tauri/tauri.conf.json`（新增 overlay 窗口：`transparent:true`/`alwaysOnTop:true`/`decorations:false`/`skipTaskbar:true`/`visible:false`）
- `src-tauri/capabilities/overlay.json`（**新，可行性#1 阻塞项**——现 `capabilities/default.json` 只授权 `main` 窗；overlay 窗 label 必须有自己的 capability 含 `core:default` + `global-shortcut:default`，否则其 webview 无法 invoke/listen）
- `src-tauri/Cargo.toml`（**加 `tauri-plugin-global-shortcut`**，版本对齐 tauri 2.5；现无此依赖）
- `src-tauri/src/lib.rs`（`.plugin(tauri_plugin_global_shortcut::Builder...)` 初始化 + 注册热键 + 显隐命令；纯 Rust 端，无需 JS 包）
**Approach**: 第二 WebviewWindow 加载 overlay 路由（U8）。**默认点击穿透**（`set_ignore_cursor_events(true)` / WS_EX_TRANSPARENT，审查 F4）——否则置顶窗吞掉游戏内点击、战斗中点技能点到浮层，直接不可用；仅在 hover 交互区或专门热键态临时接收输入。显隐切换用**不抢焦点**方式（`show`/`show_without_focus`），避免无边框游戏丢焦点/最小化（F4）。全局热键（如 Ctrl+Shift+T）切换显隐。
**Patterns to follow**: Nidalee 现有窗口配置（`tauri.conf.json` 主窗 `decorations:false`）、`capabilities/default.json`（capability 格式）、`lib.rs` 现有 `.plugin(...)` 初始化（updater/process/dialog）。
**Test scenarios**:
- Happy: 应用启动后热键切换 overlay 窗显隐；overlay webview 能 invoke `get_mayhem_champion`（验 capability 生效）。
- Edge: 重复热键快速触发不崩、状态一致。
- Test expectation: 窗口/穿透/焦点行为以真机 Windows 手动验收为主。
**Verification（Windows 活体，审查 F4）**: 无边框模式下——①浮层可见、置顶、半透明；②**战斗中浮层区域点击穿透到游戏**（非交互态）；③切显隐**不导致游戏丢焦点/最小化**；④4K/DPI 缩放下位置正确。

### U8. 浮层决策卡 UI（Vue）

**Goal**: 浮层内展示本局英雄的海克斯优先级排序 + top 三连 + 核心装，自动跟随当前英雄。
**Requirements**: R1, R4, R5
**Dependencies**: U3, U4, U7
**Files**:
- `src/views/OverlayView.vue`（新）
- `src/router`（overlay 路由）
- 复用 `src/composables/mayhem/useMayhemData.ts`（U6）
**Approach**: 精简布局：海克斯按稀有度分组（白银/黄金/棱彩）× win_rate 降序；top N 三连（图标行 + 胜率）；核心装。订阅 U4 当前英雄自动刷新。角标「KR / 版本 X」（R5）。**不含三选一自动识别**（不可能，见 Scope）。
**Patterns to follow**: U6 的 useMayhemData；shadcn-vue + tailwind（尺寸用 px 任意值，遵项目 Tailwind 约定）。
**Test scenarios**:
- Happy: 给当前英雄 → 渲染分组排序列表 + 三连 + 核心装 + 来源标签。
- Edge: 英雄切换 → 内容随之更新；无数据 → 占位态。
- Covers R4: 排序展示只由 win_rate 驱动。
**Verification**: 局内浮层显示当前英雄正确的海克斯优先级，切英雄自动更新。

### U9. 速查库（Vue）

**Goal**: 不打游戏也能搜任意英雄的海克斯大乱斗数据 + 全局海克斯 tier 表。
**Requirements**: R3
**Dependencies**: U3, U2
**Files**:
- `src/views/MayhemCodexView.vue`（新）
- `src/router` + 侧栏入口（`src/components/layout` / `src/components/ui/sidebar`）
- 复用 useMayhemData + U2 元数据
**Approach**: 英雄搜索 → 跳该英雄海克斯/出装/三连（复用 U8/U6 展示块）；全局海克斯 tier 表（可搜/排序，聚合各英雄或全局 augment 胜率）。纯复用数据，不新造源。
**Patterns to follow**: `@tanstack/vue-table`（仓库已装，用于 tier 表）、现有 view + router 结构。
**Test scenarios**:
- Happy: 搜「德邦」→ 出德邦海克斯大乱斗数据；tier 表按胜率排序。
- Edge: 搜不到 → 空态提示；搜索去抖不卡。
**Verification**: 离线（非对局）状态下可查任意英雄 + 浏览海克斯 tier 表。

### U10. 出厂数据快照 + 冷启动兜底（审查 F5，提到 v1）

**Goal**: Blitz 不可达/端点失效/冷启动缓存空时，工具仍能用出厂快照降级运行。
**Requirements**: R4, R5（可用性兜底）；直接缓解 KTD7(F1) 可达性风险
**Dependencies**: U1, U3
**Files**:
- `src-tauri/resources/mayhem-snapshot.json`（新，打包进应用的各英雄 Mayhem 数据快照）
- `src-tauri/tauri.conf.json`（`bundle.resources` 纳入快照）
- `scripts/build-mayhem-snapshot.mjs`（新，构建期批量拉 Blitz 生成快照）
- `src-tauri/src/data/blitz/mod.rs` 或 U3 取数层（读取回退逻辑）
**Approach**: 三层取数——① 实时 Blitz；② 本地成功缓存；③ **打包出厂快照**（保底）。任一层拿到即用，并在 UI 标数据「实时/缓存/出厂(版本X)」。快照由构建脚本离线批量拉 Blitz 生成（不进运行时热路径）。这比双源容灾便宜，却兜住 F1(不可达) + F5(冷启动/端点失效) 两个最致命场景。
**Patterns to follow**: Nidalee `bundle` 资源打包；`data/blitz` 取数层。
**Test scenarios**:
- Happy: 模拟 Blitz 请求失败 → 回退读出厂快照 → 返回该英雄数据 + 标「出厂」。
- Edge: 快照缺某英雄 → 明确空态，不 panic。
- Covers F5: 缓存为空(冷启动) + Blitz 失败 → 仍有出厂数据。
**Verification**: 断网/改错端点后，工具仍能显示出厂快照数据并如实标注来源。

---

## Risks & Dependencies

- **🔴 Blitz 国服可达性未验证（F1，最高优先前置 gate）**：见 KTD7。本机 Surge TUN 无法测真实直连；须 Windows+家庭网络实测。缓解：U10 出厂快照兜底。**依赖 Blitz 的功能上线前必须过这道 gate。**
- **🔴 Mayhem 是轮换/活动模式（F7）**：并非常驻。U4/U5/U6/U7 的活体验收需一台 Windows + 一局**正在开放的**海克斯大乱斗。若排期时国服未开放，「先验证再推进」的 MVP gate 卡死。缓解：数据层(U1-U3/U10)离线 fixture 可先验；item-set 写入与浮层覆盖可先用普通 ARAM/自定义局验证。
- **单源脆弱性（F5）**：非官方端点随时改查询名/限流/上 Cloudflare。缓解：U10 出厂快照兜住冷启动+端点失效（提到 v1）；真·多源交叉留 Deferred。
- **item-set schema 需真机校准（F2）**：wrapper/associatedChampions/map=12 见 KTD5，实现首步 GET 真实 sets 抓 fixture 核字段。
- **浮层点击穿透/焦点（F4）**：见 U7，默认穿透、不抢焦点，Windows 真机验三态。
- **许可证 CC BY-NC-SA 4.0**（Nidalee 基座）：衍生须**署名 + 非商用 + 同样式共享**。个人自用 OK，**不可商用/售卖**，README 须署名 Nidalee。
- **浮层仅无边框可用**（KTD2）：独占全屏盖不住。缓解：应用内提示玩家设无边框。
- **数据地区 = KR 代理**（KTD1）：国服版本滞后时绝对胜率有偏差。缓解：UI 标来源/版本，排序相对可迁移。
- **构建目标 Windows / 开发机 Mac**：Tauri 跨平台。Mac 可验：Rust 编译+单测、type-check、前端 build。须 Windows 验：活 LCU、浮层覆盖、item-set 写入、热键、Blitz 可达性。
- **外部依赖**：CommunityDragon `cherry-augments`（augment 元数据）、新增 `tauri-plugin-global-shortcut`。

---

## Scope Boundaries

### 本计划内
- 三交付物（浮层 / 出装助手 / 速查库）+ Blitz 数据层 + 出装写入 + 当前英雄识别。
- 外服 KR 数据 + UI 来源/版本标注。

### Deferred to Follow-Up Work
- 双外服源容灾（Blitz + arammayhem/op.gg fallback）。
- 深探海斗小助手微信小程序是否有国服数据。
- 「贴客户端右侧」独立停靠窗（v1 先主窗内视图）。

### 明确不做（产品边界）
- **自动识别游戏内实际给出的 3 个海克斯并替你选**——官方 API 无此数据，物理不可能。
- **阵容维度的静态 AP/AD 标签启发式**——海克斯打乱定位，标签失效（见 CONTEXT「静态标签失效」）；v2 若做须数据支撑。
- **动态补齐三连**（按已选海克斯反查下一个）——v2 拓展。
- 内存注入 / 读屏 / DirectX hook。

---

## Sources & Research

- 数据源实测（本会话）：`datalake.v2.iesdev.com/graphql` `prod_aram_mayhem_champion`，champion_id=5 返回 augments(164)/items(157)/augment_trios(248) + 胜率，免鉴权。⚠️ 均**经开发机 Surge 代理/TUN**，国服无代理可达性未验（见 KTD7/F1）。
- 元数据源实测：CommunityDragon `cherry-augments.json` 覆盖 Blitz 全部 164 augment id（100%），zh_cn 有中文名，图标 URL 变换已验 200（见 KTD4）。
- 参考工具 lolpro（`heichenya/lolpro`）：验证了 Blitz 源 + 浮层可行；**无 License，不搬代码**，只用实测端点事实。
- 官方约束：RiotGames/developer-relations #1154（ARAM Mayhem 增强数据不进官方 API）。
- 基座 Nidalee（`codeXcn/Nidalee`）：LCU 模块、符文导入、gameflow/champ-select 轮询可复用。⚠️ `types.rs::ItemSet` 是 op.gg 形状**非** LCU 写入 schema、无既有 item-set 写入（见 KTD5/F2）。
- 计划深审（本会话）：ce-adversarial + ce-feasibility 两轮审查，7+3 条发现已整合（F1-F7 + 可行性 3 条）。
