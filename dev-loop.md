# Dev Loop: 抓 issue → worktree 开发 → 验收 → PR → 人批后合入

> 与仓根 `loop.md`（运营环：bug 小修/验收债/灵感/概设）分工：运营环把需求变成
> ready 的 issue 后**停**；本环接力把 issue 变成合入的 PR。触发：`/loop 按 dev-loop.md 执行`。
>
> 设计参考开源社区方案：一轮一分支一 PR + 三硬停（continuous-claude）、一轮只做
> 一个 item + 状态落盘（Ralph loop）、不自动合入默认分支改人批门（claude-code-action）、
> label 语义分工（OpenHands resolver）。

## VISION（北极星）

把带 `enhancement` 的 open issue 逐个变成「VERIFY 全绿 + 验收证据齐全 + 经人批准
后合入 main」的 PR，直到 backlog 清空。守 docs/target-effect.md 红线与
docs/adr 决策；数据驱动，不做人工评级类功能。

## 每 tick 做什么（ACTION）——一轮严格只做一个单元

1. **读状态（必先做，再动作）**：
   - `gh pr list --label loop-dev --json number,title,reviewDecision,comments`
   - `gh issue list --state open --label enhancement --json number,title,labels`
   - 读上一轮 record（`docs/loop-records/dev-*.md` 最新一份）
   - `git -C <仓根> worktree list`（现存开发 worktree）
2. **按优先级选一个单元**（从上往下命中即做，只做这一个）：
   - **A 合入已批 PR**：某 loop-dev PR `reviewDecision=APPROVED` →
     squash merge（`gh pr merge --squash --delete-branch`）→ 确认 issue 被
     `Closes #N` 自动关闭 → `git worktree remove` 清理 → 摘 `in-dev` label。
   - **B 响应 review 意见**：某 loop-dev PR 有未解决的 changes-requested/评论 →
     回到对应 worktree 按意见最小修改 → 重跑 VERIFY → push 更新 → 回复评论。
   - **C 开发下一个 issue**：无 A/B 时，取优先序最前的未认领 issue
     （推荐顺序 #1 → #3 → #2 → #4；跳过带 `in-dev`/`blocked` 的）：
     1. 认领：issue 加 `in-dev` label（防与运营环/并行会话双抢）
     2. `git worktree add ../mayhem-helper-wt/issue-<N> -b feat/issue-<N> --no-track origin/main`
     3. worktree 内 `pnpm install`（共享 store，快）+ 从主仓拷 `types/*.d.ts`
        （unplugin 生成物整目录被 gitignore，缺了 type-check 假红一片）
     4. 全文读 issue（目标/验收 checklist/边界）+ 相关文件后实现，**最小改动**，
        不越 issue 边界；发现 issue 描述与代码现实矛盾 → 停该单元，评论 issue 升级人工
     5. 跑 VERIFY（见下）→ 全绿才 push 分支
     6. `gh pr create`：标题对齐 issue，body 含 `Closes #N` + **验收 checklist 逐项
        evidence**（named-case：每项写「怎么验的 + 结果」）+ VERIFY 命令输出摘要，
        加 `loop-dev` label → **停，等人批**（不自动合入）
3. **写 record**（见 RECORD）→ 本 tick 结束。

## VERIFY（能说不的 gate，全绿才 push/merge）

在 worktree 内依次跑，任一红即本单元失败：

- `pnpm type-check`（vue-tsc）
- `pnpm lint`（oxlint + eslint，error 必清，不用 any/eslint-disable 绕；
  lint --fix 会顺手重排未改动文件——push 前 `git status` 核 diff，无关文件一律 checkout 回滚）
- `pnpm type-check` 以 main 基线对照（main 现有 51 个历史报错）：改动文件 0 新增才算绿
- `pnpm build-only`（vite 构建成功）
- 触碰 `src-tauri/` 时追加：`cd src-tauri && cargo test --lib`
- **验收 rubric（named-case）**：issue 的验收 checklist 逐项给 evidence
  （UI 项：`pnpm dev` + 浏览器实测/截图；数据项：实际请求/快照输出摘录）。
  无 evidence 的项不得勾选，如实标「未验 + 原因」。

失败回灌：VERIFY 红 → 修一次重跑；再红 → record 里记完整报错，进无进展计数
（失败信息作为下一 tick 同单元的输入，不盲目重试）。

## 硬停

- 最大轮数：**MAX_ITER=8**（单次 /loop 会话）
- 无进展：同一 issue VERIFY 连红 **2 tick** → issue 加 `blocked` label + 评论卡点，
  换下一个 issue；**2 个 issue 同时 blocked** → 整环停，升级人工
- 待批积压：**≥2 个 PR 等 review 且无 A/B 单元可做** → 停（不堆无人看的 PR）
- 预算：每 tick ≤ 15 万 token；单日 ≤ 2 次 /loop 会话
- 需人判即停：merge 冲突需要取舍、issue 边界要扩、改动波及 CI/构建配置/lockfile
  （pnpm install 因新增依赖正常更新 lockfile 除外，需在 PR body 说明）

## SCOPE

- 管：label=`enhancement` 的 open issues（当前 #1–#4）、feat/issue-* 分支、
  loop-dev PR 的全生命周期（开发→回应 review→合入→清理）
- 不管：发版打 tag、运营环的 P0-P3 单元、无 label 的 issue、直接 push main（一律走 PR）

## REPORT

- 每 tick：record 落 `docs/loop-records/dev-YYYY-MM-DD-tick<N>.md`
- 会话收尾：控制台汇总（合入了什么 / 开了什么 PR 等批 / blocked 什么），列 PR 链接

## RECORD（审计回执）

```
单元: <A-merge#PR / B-review#PR / C-dev#issue>
发现: <findings，含读到的 review 意见/报错>
改动: <files 或 无>
VERIFY: <逐命令 pass|fail + 验收 rubric evidence 摘要>
remaining_delta: <open enhancement issues 数 / 待批 PR 数 / blocked 数>
```

收敛信号：remaining_delta 逐轮缩小；不缩触发无进展硬停。

## 并行（默认不用）

单 tick 只做一个单元、worktree 串行使用——4 个 issue 改动面重叠
（MayhemCodexView/useMayhemData），并行开发必然冲突，不并。
