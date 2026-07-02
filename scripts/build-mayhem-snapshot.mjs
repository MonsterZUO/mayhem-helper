#!/usr/bin/env node
// 构建出厂快照：批量拉 Blitz Datalake 各英雄海克斯大乱斗数据，写入
// src-tauri/resources/mayhem-snapshot.json（Blitz 不可达时的兜底，深审 F1/F5）。
//
// 用法：node scripts/build-mayhem-snapshot.mjs
// 需在能访问 datalake.v2.iesdev.com 的网络下运行（国服可能需代理）。

import { writeFileSync, mkdirSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const BLITZ_URL = 'https://datalake.v2.iesdev.com/graphql'
const CHAMPION_SUMMARY =
  'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-summary.json'
const OUT = resolve(dirname(fileURLToPath(import.meta.url)), '../src-tauri/resources/mayhem-snapshot.json')

const QUERY = `query q($champion_id: String!){executeDatabricksQuery(game: LEAGUE queryName:"prod_aram_mayhem_champion" params:[{name:"champion_id",value:$champion_id}]){payload}}`

async function fetchChampionIds() {
  const res = await fetch(CHAMPION_SUMMARY)
  const list = await res.json()
  return list.map((c) => c.id).filter((id) => id > 0)
}

async function fetchMayhem(championId) {
  const res = await fetch(BLITZ_URL, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'User-Agent': 'Mozilla/5.0' },
    body: JSON.stringify({ query: QUERY, variables: { champion_id: String(championId) } })
  })
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  const json = await res.json()
  const payload = json?.data?.executeDatabricksQuery?.payload
  const row = payload?.result?.dataArray?.[0]
  if (!row) return null
  // 列式：找 data 列位置
  const cols = payload.manifest.schema.columns
  const dataPos = cols.find((c) => c.name === 'data')?.position
  const patchPos = cols.find((c) => c.name === 'patch')?.position
  return { data: JSON.parse(row[dataPos]), patch: row[patchPos] }
}

async function main() {
  const ids = await fetchChampionIds()
  console.log(`拉取 ${ids.length} 个英雄的海克斯大乱斗数据…`)
  const champions = {}
  let patch = ''
  let ok = 0
  for (const id of ids) {
    try {
      const r = await fetchMayhem(id)
      if (r) {
        champions[id] = r.data
        patch = r.patch || patch
        ok++
      }
    } catch (e) {
      console.warn(`  英雄 ${id} 失败: ${e.message}`)
    }
    await new Promise((r) => setTimeout(r, 120)) // 温柔限速
  }
  mkdirSync(dirname(OUT), { recursive: true })
  writeFileSync(OUT, JSON.stringify({ patch, source: 'KR', champions }), 'utf8')
  console.log(`✅ 写入 ${OUT}：${ok}/${ids.length} 英雄，版本 ${patch}`)
}

main().catch((e) => {
  console.error('构建快照失败:', e)
  process.exit(1)
})
