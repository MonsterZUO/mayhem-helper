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
  /** 平均购买顺位（0.7≈起始装，越大越后期） */
  average_index: number
}

/** 鞋子物品 id（出装路线分组用） */
const BOOTS_IDS = new Set([1001, 3005, 3006, 3009, 3010, 3020, 3047, 3111, 3117, 3158, 2422])

/** op.gg 式出装路线：起始装 / 核心序列（按购买顺位）/ 鞋 / 其余选择表。 */
export interface BuildRoute {
  starters: RankedItem[]
  core: RankedItem[]
  boots: RankedItem | null
  options: RankedItem[]
}

/** 纯函数：从扁平 items 重建出装路线（average_index=购买顺位）。 */
export function buildRoute(items: RankedItem[]): BuildRoute {
  const meaningful = items.filter((i) => i.num_games >= 30)
  const byPick = [...meaningful].sort((a, b) => b.pick_rate - a.pick_rate)

  const starters = byPick.filter((i) => i.average_index < 1.3 && !BOOTS_IDS.has(i.id)).slice(0, 2)
  const bootsCandidates = byPick.filter((i) => BOOTS_IDS.has(i.id))
  const boots = bootsCandidates[0] ?? null

  const starterIds = new Set(starters.map((i) => i.id))
  // 核心三件：选取率最高的非鞋非起始大件，按购买顺位排成序列
  const core = byPick
    .filter((i) => !BOOTS_IDS.has(i.id) && !starterIds.has(i.id) && i.average_index >= 1.3)
    .slice(0, 3)
    .sort((a, b) => a.average_index - b.average_index)

  const usedIds = new Set([...starterIds, ...(boots ? [boots.id] : []), ...core.map((i) => i.id)])
  const options = [...meaningful]
    .filter((i) => !usedIds.has(i.id) && !BOOTS_IDS.has(i.id))
    .sort((a, b) => b.win_rate - a.win_rate)
    .slice(0, 8)

  return { starters, core, boots, options }
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
export function groupAugmentsByRarity(
  augments: RankedAugment[]
): Array<{ rarity: AugmentRarity; label: string; items: RankedAugment[] }> {
  return RARITY_ORDER.map((rarity) => ({
    rarity,
    label: RARITY_LABEL[rarity],
    items: augments.filter((a) => a.rarity === rarity)
  })).filter((group) => group.items.length > 0)
}

/** 物品图标 URL（腾讯国服 CDN，国内访问快）。 */
export function itemIconUrl(itemId: number): string {
  return `https://game.gtimg.cn/images/lol/act/img/item/${itemId}.png`
}

/** 物品 id→中文名映射（ddragon zh_CN，进程内缓存）。 */
export function useItemNames() {
  return useQuery({
    queryKey: ['item-names-zh'],
    queryFn: async () => {
      const { fetchItems } = await import('@/lib/dataApi')
      const res = await fetchItems()
      const map = new Map<number, string>()
      const data = (res.data as { data?: Record<string, { name?: string }> })?.data ?? {}
      for (const [id, item] of Object.entries(data)) {
        if (item?.name) map.set(Number(id), item.name)
      }
      return map
    },
    staleTime: Infinity
  })
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

/** 全局海克斯榜条目（跨英雄加权聚合）。 */
export interface GlobalAugment {
  id: number
  name: string
  icon_url: string
  rarity: AugmentRarity
  win_rate: number
  num_games: number
  champion_count: number
}

export interface MayhemAugmentTiers {
  patch: string
  source: string
  augments: GlobalAugment[]
}

/** 响应式取全局海克斯榜（本地快照聚合，秒回）。 */
export function useMayhemAugmentTiers(enabled: MaybeRefOrGetter<boolean>) {
  return useQuery({
    queryKey: ['mayhem-augment-tiers'],
    queryFn: () => invoke<MayhemAugmentTiers>('get_mayhem_augment_tiers'),
    enabled: computed(() => toValue(enabled)),
    staleTime: Infinity
  })
}

/** 海克斯详情里的单英雄战绩条目。 */
export interface AugmentChampionEntry {
  champion_id: number
  win_rate: number
  pick_rate: number
  num_games: number
}

/** 海克斯详情（反向索引：海克斯 → 适配英雄排行，按胜率降序）。 */
export interface MayhemAugmentDetail {
  id: number
  name: string
  icon_url: string
  rarity: AugmentRarity
  patch: string
  source: string
  /** 入榜的单英雄最小局数门槛 */
  min_games: number
  /** 因样本不足被过滤的英雄数 */
  filtered_out: number
  champions: AugmentChampionEntry[]
}

/** 响应式取某海克斯的详情（快照反向聚合，秒回）。 */
export function useMayhemAugmentDetail(augmentId: MaybeRefOrGetter<number | null>) {
  const aid = computed(() => toValue(augmentId))
  return useQuery({
    queryKey: ['mayhem-augment-detail', aid],
    queryFn: () => invoke<MayhemAugmentDetail>('get_mayhem_augment_detail', { augmentId: aid.value }),
    enabled: computed(() => aid.value != null && aid.value > 0),
    staleTime: Infinity
  })
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
