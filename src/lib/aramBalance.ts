import aramBalanceData from '@/assets/aram-balance.json'

/**
 * ARAM 平衡参数（海克斯大乱斗基于嚎哭深渊，沿用 ARAM 系数）。
 * 数据由 scripts/build-aram-balance.mjs 从 LoL Wiki 生成，静态打包。
 * 字段除 ability_haste 为加数（±N）外均为乘数（1=无调整，生成时已剔除）。
 */
interface AramBalanceFile {
  source: string
  fetched: string
  champions: Record<string, Record<string, number>>
}

const data = aramBalanceData as AramBalanceFile

/** 字段 → 中文标签（顺序即展示顺序，伤害两项最关键放前） */
const FIELD_LABELS: Array<{ key: string; label: string }> = [
  { key: 'dmg_dealt', label: '造成伤害' },
  { key: 'dmg_taken', label: '承受伤害' },
  { key: 'healing', label: '治疗效果' },
  { key: 'shielding', label: '护盾效果' },
  { key: 'attack_speed', label: '攻击速度' },
  { key: 'ability_haste', label: '技能急速' },
  { key: 'tenacity', label: '韧性' },
  { key: 'energy_regen', label: '能量回复' }
]

export interface AramBalanceEntry {
  label: string
  /** 展示文本，如 "-10%"、"+20" */
  text: string
  /** 对英雄是否增益（决定展示色） */
  buff: boolean
}

/** dmg_taken 数值变小是增益，其余变大是增益 */
const LOWER_IS_BUFF = new Set(['dmg_taken'])

/** 某英雄的 ARAM 平衡调整列表；无调整返回空数组（前端不渲染区块）。 */
export function aramBalanceEntries(championId: number): AramBalanceEntry[] {
  const raw = data.champions[String(championId)]
  if (!raw) return []
  const entries: AramBalanceEntry[] = []
  for (const { key, label } of FIELD_LABELS) {
    const v = raw[key]
    if (v === undefined) continue
    if (key === 'ability_haste') {
      entries.push({ label, text: `${v > 0 ? '+' : ''}${v}`, buff: v > 0 })
    } else {
      const pct = Math.round((v - 1) * 1000) / 10
      entries.push({
        label,
        text: `${pct > 0 ? '+' : ''}${pct}%`,
        buff: LOWER_IS_BUFF.has(key) ? v < 1 : v > 1
      })
    }
  }
  return entries
}

/** 数据来源说明（角标用） */
export const ARAM_BALANCE_SOURCE = `${data.source} · ${data.fetched}`
