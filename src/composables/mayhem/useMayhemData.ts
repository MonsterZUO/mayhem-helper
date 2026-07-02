import { invoke } from '@tauri-apps/api/core'
import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'

// 与 Rust common::commands::mayhem 的 serde 输出对应
export type AugmentRarity = 'silver' | 'gold' | 'prismatic' | 'unknown'

export interface RankedAugment {
  id: number
  name: string
  icon_url: string
  rarity: AugmentRarity
  win_rate: number
  pick_rate: number
  num_games: number
}

export interface RankedItem {
  id: number
  win_rate: number
  pick_rate: number
  num_games: number
}

export interface AugmentTrio {
  ids: [number, number, number]
  names: [string, string, string]
  /** 胜率档位，1=最优、5=最差 */
  win_rate_tier: number
  pick_rate_tier: number
  num_games: number
}

export interface MayhemChampion {
  champion_id: number
  patch: string
  /** 数据来源标识，如 "KR" */
  source: string
  win_rate: number
  pick_rate: number
  augments: RankedAugment[]
  core_items: RankedItem[]
  trios: AugmentTrio[]
}

const RARITY_ORDER: AugmentRarity[] = ['prismatic', 'gold', 'silver', 'unknown']
const RARITY_LABEL: Record<AugmentRarity, string> = {
  prismatic: '棱彩',
  gold: '黄金',
  silver: '白银',
  unknown: '未知'
}

export function rarityLabel(rarity: AugmentRarity): string {
  return RARITY_LABEL[rarity]
}

/** 按稀有度分组（棱彩→黄金→白银），组内保持传入的胜率降序。 */
export function groupAugmentsByRarity(augments: RankedAugment[]): Array<{ rarity: AugmentRarity; label: string; items: RankedAugment[] }> {
  return RARITY_ORDER.map((rarity) => ({
    rarity,
    label: RARITY_LABEL[rarity],
    items: augments.filter((a) => a.rarity === rarity)
  })).filter((group) => group.items.length > 0)
}

/** 物品图标 URL（CommunityDragon）。 */
export function itemIconUrl(itemId: number): string {
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/assets/items/icons2d/${itemId}.png`
}

export function fetchMayhemChampion(championId: number): Promise<MayhemChampion> {
  return invoke<MayhemChampion>('get_mayhem_champion', { championId })
}

/** 取当前对局本地玩家英雄 id（未在选英雄/未分配时为 null）。 */
export function getCurrentChampionId(): Promise<number | null> {
  return invoke<number | null>('get_current_champion_id')
}

/** 把核心出装写入客户端预设出装。 */
export function applyMayhemItemSet(championId: number, itemIds: number[]): Promise<string> {
  return invoke<string>('apply_mayhem_item_set', { championId, itemIds })
}

/** 切换游戏内浮层显隐。 */
export function toggleOverlay(): Promise<void> {
  return invoke<void>('toggle_overlay_cmd')
}

/** 响应式取某英雄的海克斯大乱斗推荐。 */
export function useMayhemChampion(championId: MaybeRefOrGetter<number | null>) {
  const cid = computed(() => toValue(championId))
  return useQuery({
    queryKey: ['mayhem-champion', cid],
    queryFn: () => fetchMayhemChampion(cid.value as number),
    enabled: computed(() => cid.value != null && cid.value > 0),
    staleTime: 1000 * 60 * 30
  })
}
