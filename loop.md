# Loop: mayhem-helper 仓库运营（issue 处理 · 验收 · 功能灵感 · 概设）

## VISION（北极星）
把 mayhem-helper 持续推向「国服海克斯大乱斗玩家可靠可用」：用户报的 bug 被修掉、
验收债（docs/self-acceptance.md 的 ⏳/📋 项）被逐项清掉、新功能以「调研→HTML 概设→issue 存档」
的节奏有序进入 backlog——而不是无序堆代码。
约束：守 docs/target-effect.md 红线（不注入/不读屏/不替选三选一/KR 数据如实标注）；
基座 CC BY-NC-SA 4.0 非商用。

## 每 tick 做什么（ACTION）
每 tick 只处理 **一个最小单元**，按优先级选取：

1. **读状态（必先做，再动作）**：
   - `git -C <仓根> status --short` + `git log --oneline -3`（工作树干净才动）
   - `gh issue list --state open --json number,title,labels`（新 bug / 待办功能）
   - 读上一轮 record（`docs/loop-records/` 最新一份）的 remaining_delta
   - `ping -c1 -W2 192.168.71.9`（家中 Windows 是否可达，决定验收流可用性）
2. **按优先级选一个单元**：
   - **P0 修 bug**：open issue 中 label=bug（或用户描述明显是缺陷）→ 复现→最小修复→VERIFY 绿→自动 commit+push→issue 回复修复说明并关闭。
   - **P1 清验收债**：self-acceptance.md 中 ⏳/📋 项，挑当前条件能验的一项——
     Windows 可达→SSH 跑 `scripts/windows-acceptance.ps1`（LCU/出装写入活体项）；
     不可达→GUI 项走本地 dev+浏览器 mock 截图审查（视觉 rubric）。
     发现缺陷：小者直接修（走 P0 流程），大者 `gh issue create` 提 bug 存档。
     验过的项更新 self-acceptance.md 状态。
   - **P2 收功能灵感**（低频：backlog 空或距上次 ≥7 天才做）：扫 arammayhem/u.gg/
     mobalytics 的 Mayhem 功能面 + Blitz 数据能力（augment_trios 等未用字段），
     过三判 rubric 后精选 ≤2 个，`gh issue create` label=feature-idea 存档（一句话价值+数据依据）。
   - **P3 新功能概设**（仅当用户点名某 feature-idea，或 issue 被标 label=concept-wanted）：
     完整调研（数据可行性实测+竞品对照+与现架构的接缝）→ 写 **HTML 完整概设**
     （自包含单文件，落 `docs/concepts/<slug>.html`，含：问题/数据依据/交互草图/
     技术方案/改动面/验收标准）→ commit+push → 在对应 issue 贴概设链接，label 改
     concept-ready → **停**。后续由用户完善成可执行方案、开 worktree 开发，loop 不擅自实现。
3. **跑 VERIFY**（见下），绿→写 record→本 tick 结束；红→修一次，再红记录并进无进展计数。

## VERIFY（能说不的 gate，按单元类型）
- **代码改动（P0/P1 小修）**：`cd src-tauri && cargo test --lib`（96+ 全绿）
  **且** `pnpm build-only` 成功——**两者都绿才允许 push**；红则不推。
- **Windows 活体验收（P1）**：SSH 跑 `scripts/windows-acceptance.ps1`，按输出行判
  PASS/FAIL（named-case：F1_Blitz / LCU_summoner / U4_champselect / U5_itemsets 逐项 evidence）。
- **GUI 视觉验收（P1）**：dev server + Tauri mock + 浏览器截图，rubric 三判：
  ①无裂图/空白 ②信息层次清晰（分档/胜率/来源标可辨）③操作可达（按钮可见可点）。
- **功能灵感（P2）三判 rubric**：①Blitz/cdragon 数据真支持（实测字段存在）
  ②不越 target-effect.md 红线 ③与现有 issues/backlog 不重复——三判全过才准立 issue。
- **概设（P3）**：HTML 文件存在且含六必备节 + issue 已建/已更新 + 链接可达。

## 硬停
- 最大轮数：**MAX_ITER=5**（单次 /loop 会话）
- 无进展：同一失败/空产出 **连 2 轮** → 停并升级人工（record 里写明卡点）
- 预算：**每 tick ≤ 10 万 token**（重调研的 P3 tick ≤ 20 万）；单日 ≤ 3 次 /loop 会话
- 需人判即停：发版(tag)、新功能进入开发、删除任何数据、改动边界超出单个单元 → 一律停下报告，不自动做

## SCOPE
- 管：本仓 issues、docs/self-acceptance.md 验收项、docs/concepts/ 概设、功能 backlog(issues)
- 不管：发版打 tag（人批）、上游 Nidalee 功能面（对局分析/自动接受等基座功能）、
  Windows 侧环境配置、任何越红线的功能（读屏/注入类灵感直接否决不立项）

## REPORT
- 每 tick：record 落 `docs/loop-records/YYYY-MM-DD-tick<N>.md`（见 RECORD）
- 会话收尾：控制台汇总（处理了什么/剩什么/建议下次做什么），有 push 则列 commit

## RECORD（审计回执）
每轮 Write 一份 record：
```
单元: <P0-issue#N / P1-验收项X / P2-灵感 / P3-概设slug>
发现: <findings>
改动: <files 或 无>
VERIFY: <pass|fail + 证据行>
remaining_delta: <open bugs 数 / 未验收项数 / backlog 数>
```
收敛信号：remaining_delta 逐轮缩小；不缩即触发无进展硬停。

## 并行（可选）
P1 验收里「SSH 活体四项」与「GUI 截图审查」相互独立，可 TaskCreate 拆两个后台子任务并行收结果；其余串行。
