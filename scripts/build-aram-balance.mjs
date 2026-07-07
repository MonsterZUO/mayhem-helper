#!/usr/bin/env node
// 抓取 LoL Wiki (Fandom) ChampionData 模块，提取各英雄 ARAM 平衡参数，
// 生成 src/assets/aram-balance.json（静态打包，前端英雄头卡展示用）。
// 用法：node scripts/build-aram-balance.mjs
// 数据说明：海克斯大乱斗基于嚎哭深渊，沿用 ARAM 平衡系数（乘数，1=无调整）。

import { writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const MODULE_API =
  'https://leagueoflegends.fandom.com/api.php?action=query&prop=revisions&titles=Module%3AChampionData%2Fdata&rvslots=main&rvprop=content&format=json&formatversion=2'
// Fandom 拦无 UA 的请求，带浏览器 UA + Accept 即通
const HEADERS = {
  'User-Agent':
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36',
  Accept: 'application/json'
}

/** 从单个英雄的 Lua 块提取 aram 子表字段（["dmg_dealt"] = 0.9 形式）。 */
function parseAramTable(championBlock) {
  const m = championBlock.match(/\["aram"\]\s*=\s*\{([^}]*)\}/)
  if (!m) return null
  const out = {}
  for (const [, key, val] of m[1].matchAll(/\["(\w+)"\]\s*=\s*(-?[\d.]+)/g)) {
    out[key] = Number(val)
  }
  return Object.keys(out).length ? out : null
}

const res = await fetch(MODULE_API, { headers: HEADERS })
if (!res.ok) throw new Error(`Fandom API HTTP ${res.status}`)
const lua = (await res.json()).query.pages[0].revisions[0].slots.main.content

// 按英雄块切分：顶层条目形如 `\n  ["英雄名"] = {`（两空格缩进），嵌套字段缩进更深不会误切
const blocks = lua.split(/\n  \["/).slice(1)
const balance = {}
let withAdjust = 0
for (const block of blocks) {
  const idMatch = block.match(/\["id"\]\s*=\s*(\d+)/)
  if (!idMatch) continue
  const aram = parseAramTable(block)
  if (!aram) continue
  // 全部为 1 的（无调整）不收录，前端「无调整不显示」直接靠缺 key 实现
  const meaningful = Object.entries(aram).filter(([, v]) => v !== 1)
  if (!meaningful.length) continue
  balance[idMatch[1]] = Object.fromEntries(meaningful)
  withAdjust++
}

if (withAdjust < 50) {
  // 正常有百余英雄带 ARAM 调整，数量骤降说明上游格式变了——fail loud，不写残缺文件
  throw new Error(`仅解析到 ${withAdjust} 个英雄的 ARAM 调整，疑似 wiki 模块格式变更，中止`)
}

const outPath = join(dirname(fileURLToPath(import.meta.url)), '../src/assets/aram-balance.json')
writeFileSync(
  outPath,
  JSON.stringify({ source: 'LoL Wiki (Fandom) Module:ChampionData', fetched: new Date().toISOString().slice(0, 10), champions: balance }, null, 2) + '\n'
)
console.log(`✅ aram-balance.json：${withAdjust} 个英雄带 ARAM 平衡调整`)
