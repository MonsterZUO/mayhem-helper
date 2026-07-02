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

---

## Key Technical Decisions

- **KTD1 数据源 = Blitz Datalake 外服代理**（见 [ADR-0001](../adr/0001-data-source-blitz-kr-proxy.md)）。`datalake.v2.iesdev.com/graphql`，`prod_aram_mayhem_champion` 查询按 champion_id 返回 augments/items/augment_trios（含胜率/选取率/tier/rank）。实测免鉴权直连、返回 Databricks 列式 payload。
- **KTD2 浮层 = OS 置顶窗，非注入**（见 [ADR-0002](../adr/0002-overlay-topmost-window-not-injection.md)）。Tauri 第二窗口 transparent + alwaysOnTop + decorations:false，游戏须无边框模式。热键呼出/收起用 Tauri global-shortcut。
- **KTD3 基座 = 续用 Nidalee**（见 [ADR-0003](../adr/0003-base-on-nidalee.md)）。复用 LCU/对局分析/符文导入；Blitz 查询层自研 Rust（lolpro 无 License，只借鉴实测得到的端点事实，不搬代码）。
- **KTD4 海克斯 id→名称/图标**：Blitz 只给 augment id（如 2095/1401）。展示需 id→name/icon/rarity 映射，取 CommunityDragon arena 数据（`raw.communitydragon.org`，出装页图标源已在用）。
- **KTD5 出装写入端点**：`POST /lol-item-sets/v1/item-sets/{summonerId}/sets`。summonerId 走 `/lol-summoner/v1/current-summoner`（Nidalee 已有 summoner service）。`ItemSet`/`ItemBlock` 类型 `src-tauri/src/lcu/types.rs` 已定义，复用。
- **KTD6 命令注册**：新 Tauri command 统一在 `src-tauri/src/lib.rs` 的 `generate_handler!` 注册，沿仓库 `lcu::<mod>::commands::<fn>` 惯例。

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
- `src-tauri/src/data/augments/metadata.rs`（新，拉取/缓存 CommunityDragon arena augments）
- `src-tauri/src/data/augments/metadata.rs` 内测试
**Approach**: 取 CommunityDragon arena augments 数据（含 id/name/desc/iconLarge），构建 `HashMap<u32, AugmentMeta>`。item id→名称/图标复用 Nidalee 现有 ddragon（`src-tauri/src/lcu/ddragon.rs`）。稀有度：白银/黄金/棱彩由 augment 数据的 rarity 字段或 Blitz `tier` 字段确定——**实现时先校准 1-2 个已知 augment id（如 2095=棱彩S级）确认字段含义再外推**。
**Patterns to follow**: `src-tauri/src/lcu/ddragon.rs`（静态资源拉取+缓存）。
**Test scenarios**:
- Happy: 给定 fixture，`resolve(2095)` 返回非空 name + icon URL + rarity。
- Edge: 未知 id → 返回占位（id 字符串 + 默认图标），不报错。
**Verification**: 单测通过；已知 augment id 解析出正确中文名。

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
- `src-tauri/src/lcu/champ_select/service.rs` / `gameflow/service.rs`（扩展，读当前英雄 id）
- `src-tauri/src/lcu/champ_select/commands.rs`（新命令或事件）
- `src-tauri/src/lib.rs`（注册）
- `src/composables/game/`（前端订阅当前英雄变化，复用现有轮询）
**Approach**: 复用 Nidalee 的 `unified_polling` / gameflow 轮询。选英雄阶段读 champ-select session 的本地玩家 championId；游戏内读 live game / gameflow 当前英雄。英雄变化时发事件，前端据此调 U3。**先确认 champ-select session 里本地玩家 championId 的取法**（参考 `get_champ_select_session_typed`）。
**Patterns to follow**: `src-tauri/src/lcu/champ_select/`（session 读取）、`src-tauri/src/lcu/unified_polling.rs`（事件轮询）。
**Test scenarios**:
- Happy: fixture champ-select session → 提取本地玩家 championId 正确。
- Edge: 未选英雄（championId=0）→ 返回 None，不误触发推荐。
- Integration: 英雄从 A 换成 B → 发出的当前英雄事件从 A 变 B。
**Verification（含最小闭环）**: 客户端在海克斯大乱斗选英雄/局内时，程序自动打印「当前英雄的 top 海克斯 + top 三连 + 核心装」。**此即 Phase A+B 最小闭环验收点——数据对味后再进 Phase C。**

### U5. LCU 出装（item-set）写入

**Goal**: 把当前英雄的海克斯大乱斗核心出装写成客户端预设出装，局内商店可见。
**Requirements**: R2
**Dependencies**: U3
**Files**:
- `src-tauri/src/lcu/item_sets/mod.rs`（新）
- `src-tauri/src/lcu/item_sets/service.rs`（新，构建 ItemSet + PUT/POST）
- `src-tauri/src/lcu/item_sets/commands.rs`（新）
- `src-tauri/src/lcu/mod.rs` + `src-tauri/src/lib.rs`（挂载 + 注册）
**Approach**: 取 summonerId（`/lol-summoner/v1/current-summoner`，复用 `summoner::service`）→ 用 U3 的 items 构建 `ItemSet`（`title`/`champion`/`mode:"CHERRY"或对应`/`blocks`，类型已在 `types.rs`）→ `PUT /lol-item-sets/v1/item-sets/{summonerId}/sets`（读现有 sets、追加/替换本工具生成的 set、整体回写）。**注意 item-sets 端点是整份 sets 回写，非单条 append——须先 GET 现有再合并，避免覆盖用户自定义**（守持久状态：先读后写、不裸覆盖）。
**Patterns to follow**: `src-tauri/src/lcu/perks/service.rs::apply_rune_build`（LCU 写入 + 端点封装）；`src-tauri/src/lcu/summoner/service.rs`（取 summoner）。
**Test scenarios**:
- Happy: 给定 items + summonerId，构建的 ItemSet body 结构合法（title/champion/blocks 齐全）。
- Edge: 已有同名工具 set → 替换而非重复追加；用户自定义 set 保留不丢。
- Error: summonerId 取不到（未登录）→ 返回明确错误，不写入。
**Verification**: 命令执行后，客户端局内商店出现导入的出装；重复导入不产生重复 set、不覆盖用户既有 set。

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
- `src-tauri/src/tauri.conf.json`（新增 overlay 窗口：`transparent:true`/`alwaysOnTop:true`/`decorations:false`/`skipTaskbar:true`/`visible:false`）
- `src-tauri/src/app.rs` 或窗口管理处（创建/显隐 overlay 窗）
- `src-tauri/Cargo.toml` + 插件初始化（如需 `tauri-plugin-global-shortcut`）
- `src-tauri/src/lib.rs`（注册热键 + 显隐命令）
**Approach**: 第二 WebviewWindow 加载 overlay 路由（U8）。全局热键（如 Ctrl+Shift+T）切换显隐（对齐 KTD2）。**Windows 目标须验证：无边框游戏下置顶窗可覆盖显示**（Risks 有记）。
**Patterns to follow**: Nidalee 现有窗口配置（`tauri.conf.json` 主窗 `decorations:false`）、tauri 插件初始化惯例（`src-tauri/src/lib.rs` 已用多个 `@tauri-apps/plugin-*`）。
**Test scenarios**:
- Happy: 应用启动后热键切换 overlay 窗显隐。
- Edge: 重复热键快速触发不崩、状态一致。
- Test expectation: 窗口行为以手动验收为主（无边框覆盖需真机 Windows 验证）。
**Verification**: Windows 无边框模式下，游戏中按热键浮层可见、置顶、半透明、可交互。

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

---

## Risks & Dependencies

- **许可证 CC BY-NC-SA 4.0**（Nidalee 基座）：衍生须**署名 + 非商用 + 同样式共享（继续 CC BY-NC-SA）**。个人自用 OK，**不可商用/售卖**，README 须署名 Nidalee。
- **Blitz Datalake 非官方**：端点/查询名可能变更或限流。缓解：请求失败 fail-loud + 缓存兜底；双外服源容灾（arammayhem/op.gg）列入 Deferred。
- **浮层仅无边框可用**（KTD2）：独占全屏盖不住。缓解：应用内提示玩家设无边框；Windows 真机验证覆盖行为。
- **数据地区 = KR 代理**（KTD1）：国服版本滞后时绝对胜率有偏差。缓解：UI 标来源/版本，排序相对可迁移。
- **构建目标 Windows / 开发机 Mac**：Tauri 跨平台，但浮层/热键/LCU 行为须在 Windows 验证。缓解：Windows 上打包（仓库已有 `build-msi.sh`）+ 真机测浮层。
- **海克斯 id→元数据覆盖**（KTD4）：CommunityDragon arena 数据须覆盖 Mayhem augment id。缓解：U2 先校准已知 id 再外推；未知 id 占位不崩。
- **外部依赖**：CommunityDragon（augment 元数据）、可能新增 `tauri-plugin-global-shortcut`。

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

- 数据源实测（本会话）：`datalake.v2.iesdev.com/graphql` `prod_aram_mayhem_champion`，champion_id=5 返回 augments(164)/items(157)/augment_trios(248) + 胜率，免鉴权。
- 参考工具 lolpro（`heichenya/lolpro`）：验证了 Blitz 源 + 浮层可行；**无 License，不搬代码**，只用实测端点事实。
- 官方约束：RiotGames/developer-relations #1154（ARAM Mayhem 增强数据不进官方 API）。
- 基座 Nidalee（`codeXcn/Nidalee`）：LCU 模块、符文导入、gameflow/champ-select 轮询、ItemSet 类型已备。
