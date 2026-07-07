<script setup lang="ts">
import { computed, ref } from 'vue'
import { ChevronDown, ChevronRight, MoveRight } from 'lucide-vue-next'
import {
  itemIconUrl,
  buildRoute,
  useItemNames,
  type MayhemChampion,
  type AugmentRarity
} from '@/composables/mayhem/useMayhemData'

const props = defineProps<{
  data: MayhemChampion
  /** 紧凑模式（浮层用），减小间距与条目数。 */
  compact?: boolean
  /** 隐藏来源/版本角标（外部已展示时用，避免重复）。 */
  hideMeta?: boolean
}>()

const route = computed(() => buildRoute(props.data.core_items))
const topTrios = computed(() => props.data.trios.slice(0, props.compact ? 3 : 5))
const { data: itemNames } = useItemNames()

// 海克斯三列并排(每列一个档位——实战三选一同档位, 在档内比较)；列内按胜率, top N + 统一展开
const PREVIEW_COUNT = computed(() => (props.compact ? 5 : 10))
const augExpanded = ref(false)
const RARITY_COLS: Array<{ key: AugmentRarity; label: string }> = [
  { key: 'prismatic', label: '棱彩' },
  { key: 'gold', label: '黄金' },
  { key: 'silver', label: '白银' }
]
// 海克斯中文名过滤（速查场景：只看某个海克斯在本英雄的胜率）
const augSearch = ref('')
const augmentColumns = computed(() => {
  const s = augSearch.value.trim().toLowerCase()
  return RARITY_COLS.map((col) => ({
    ...col,
    items: props.data.augments.filter((a) => a.rarity === col.key && (!s || a.name.toLowerCase().includes(s)))
  }))
})
const maxColLength = computed(() => Math.max(0, ...augmentColumns.value.map((c) => c.items.length)))

function itemName(id: number): string {
  return itemNames.value?.get(id) ?? String(id)
}

function pct(v: number): string {
  return `${(v * 100).toFixed(1)}%`
}

const rarityDot: Record<string, string> = {
  棱彩: 'bg-[#a21caf] dark:bg-[#e04ba0]',
  黄金: 'bg-[#a16207] dark:bg-[#e0a72c]',
  白银: 'bg-[#64748b] dark:bg-[#9aa4b2]',
  未知: 'bg-[#6b7280]'
}
const rarityHead: Record<string, string> = {
  棱彩: 'text-[#a21caf] dark:text-[#e04ba0]',
  黄金: 'text-[#a16207] dark:text-[#e0a72c]',
  白银: 'text-[#64748b] dark:text-[#9aa4b2]'
}
</script>

<template>
  <div class="flex flex-col" :class="compact ? 'gap-[12px]' : 'gap-[20px]'">
    <!-- 来源 / 版本 角标（数据地区代理透明标注，见 ADR-0001） -->
    <div v-if="!hideMeta" class="flex items-center justify-between text-[11px] text-foreground/50">
      <span>数据来源 {{ data.source }} · 版本 {{ data.patch }}</span>
      <span>胜率 {{ pct(data.win_rate) }} · 登场 {{ pct(data.pick_rate) }}</span>
    </div>

    <!-- ═══ 出装路线（op.gg 式：起始 → 核心序列 → 鞋） ═══ -->
    <section v-if="route.core.length || route.starters.length" class="flex flex-col gap-[8px]">
      <div class="text-[13px] font-[600] text-foreground/80">出装路线</div>
      <div
        class="flex flex-wrap items-center gap-x-[18px] gap-y-[10px] rounded-[10px] bg-accent/30 px-[12px] py-[10px]"
      >
        <div v-if="route.starters.length" class="flex items-center gap-[8px]">
          <span class="text-[11px] text-muted-foreground">起始</span>
          <div class="flex gap-[4px]">
            <img
              v-for="it in route.starters"
              :key="it.id"
              :src="itemIconUrl(it.id)"
              :alt="itemName(it.id)"
              :title="`${itemName(it.id)} · 胜率 ${pct(it.win_rate)}`"
              class="h-[30px] w-[30px] rounded-[6px] border border-border/50"
              loading="lazy"
            />
          </div>
        </div>
        <div v-if="route.core.length" class="flex items-center gap-[8px]">
          <span class="text-[11px] text-muted-foreground">核心</span>
          <div class="flex items-center gap-[4px]">
            <template v-for="(it, i) in route.core" :key="it.id">
              <MoveRight v-if="i > 0" class="h-[13px] w-[13px] text-muted-foreground/60" />
              <div class="flex flex-col items-center gap-[2px]">
                <img
                  :src="itemIconUrl(it.id)"
                  :alt="itemName(it.id)"
                  :title="`${itemName(it.id)} · 胜率 ${pct(it.win_rate)} · 顺位 ${it.average_index.toFixed(1)}`"
                  class="h-[38px] w-[38px] rounded-[7px] border border-border/50"
                  loading="lazy"
                />
                <span class="text-[10px] tabular-nums text-muted-foreground">{{ pct(it.win_rate) }}</span>
              </div>
            </template>
          </div>
        </div>
        <div v-if="route.boots" class="flex items-center gap-[8px]">
          <span class="text-[11px] text-muted-foreground">鞋</span>
          <img
            :src="itemIconUrl(route.boots.id)"
            :alt="itemName(route.boots.id)"
            :title="`${itemName(route.boots.id)} · 胜率 ${pct(route.boots.win_rate)}`"
            class="h-[30px] w-[30px] rounded-[6px] border border-border/50"
            loading="lazy"
          />
        </div>
      </div>
    </section>

    <!-- 装备选择表（非 compact 才显示） -->
    <section v-if="!compact && route.options.length" class="flex flex-col gap-[6px]">
      <div class="text-[13px] font-[600] text-foreground/80">其他高胜率装备</div>
      <ul class="grid grid-cols-1 gap-[3px] sm:grid-cols-2">
        <li
          v-for="it in route.options"
          :key="it.id"
          class="flex items-center gap-[8px] rounded-[7px] px-[8px] py-[4px] hover:bg-accent/40"
        >
          <img
            :src="itemIconUrl(it.id)"
            :alt="itemName(it.id)"
            class="h-[26px] w-[26px] rounded-[5px] border border-border/40"
            loading="lazy"
          />
          <span class="flex-1 truncate text-[12px] text-foreground/85">{{ itemName(it.id) }}</span>
          <span class="text-[12px] font-[500] tabular-nums text-foreground/80">{{ pct(it.win_rate) }}</span>
        </li>
      </ul>
    </section>

    <!-- ═══ 海克斯：三列并排(棱彩|黄金|白银)，列内按胜率——实战三选一同档位 ═══ -->
    <section class="flex flex-col gap-[6px]">
      <div class="flex items-center justify-between gap-[10px]">
        <div class="text-[13px] font-[600] text-foreground/80">海克斯排行（胜率 · 选取率）</div>
        <input
          v-if="!compact"
          v-model="augSearch"
          placeholder="搜索海克斯…"
          class="h-[26px] w-[160px] rounded-[7px] border border-border/50 bg-background/50 px-[8px] text-[12px] text-foreground outline-none transition focus:border-primary/50"
        />
      </div>
      <div class="grid grid-cols-3" :class="compact ? 'gap-[8px]' : 'gap-[14px]'">
        <div v-for="col in augmentColumns" :key="col.key" class="flex min-w-0 flex-col gap-[4px]">
          <div class="flex items-center gap-[5px] border-b border-border/50 pb-[4px]">
            <span class="h-[8px] w-[8px] rounded-full" :class="rarityDot[col.label]" />
            <span class="text-[12.5px] font-[600]" :class="rarityHead[col.label]">{{ col.label }}</span>
            <span class="text-[11px] text-muted-foreground">{{ col.items.length }}</span>
          </div>
          <div
            v-for="aug in augExpanded || augSearch.trim() ? col.items : col.items.slice(0, PREVIEW_COUNT)"
            :key="aug.id"
            class="flex items-center gap-[6px] rounded-[6px] px-[4px] py-[3px] hover:bg-accent/40"
            :title="`${aug.name} · 胜率 ${pct(aug.win_rate)} · 选取 ${pct(aug.pick_rate)}`"
          >
            <img
              v-if="aug.icon_url"
              :src="aug.icon_url"
              :alt="aug.name"
              class="h-[20px] w-[20px] shrink-0 rounded-[4px]"
              loading="lazy"
            />
            <span class="min-w-0 flex-1 truncate text-[12px] text-foreground/90">{{ aug.name }}</span>
            <span class="shrink-0 text-[12px] font-[500] tabular-nums text-foreground/90">{{ pct(aug.win_rate) }}</span>
            <span v-if="!compact" class="hidden shrink-0 text-[10.5px] tabular-nums text-muted-foreground lg:inline">{{
              pct(aug.pick_rate)
            }}</span>
          </div>
        </div>
      </div>
      <button
        v-if="maxColLength > PREVIEW_COUNT"
        class="flex items-center gap-[3px] self-start rounded px-[6px] py-[2px] text-[11px] text-muted-foreground hover:bg-accent/40 hover:text-foreground"
        @click="augExpanded = !augExpanded"
      >
        <component :is="augExpanded ? ChevronDown : ChevronRight" class="h-[12px] w-[12px]" />
        {{ augExpanded ? '收起' : '展开全部' }}
      </button>
    </section>

    <!-- ═══ 最优三连套路 ═══ -->
    <section v-if="topTrios.length" class="flex flex-col gap-[5px]">
      <div class="text-[13px] font-[600] text-foreground/80">最优三连套路</div>
      <ul class="flex flex-col gap-[3px]">
        <li
          v-for="(trio, i) in topTrios"
          :key="i"
          class="flex items-center gap-[8px] rounded-[7px] bg-accent/25 px-[8px] py-[5px]"
        >
          <span
            class="rounded-[4px] px-[6px] text-[11px] font-[600]"
            :class="trio.win_rate_tier === 1 ? 'bg-primary/20 text-primary' : 'bg-accent text-muted-foreground'"
            >T{{ trio.win_rate_tier }}</span
          >
          <span class="flex-1 truncate text-[12.5px] text-foreground/85">{{ trio.names.join('  +  ') }}</span>
          <span class="text-[11px] tabular-nums text-muted-foreground">{{ trio.num_games }} 局</span>
        </li>
      </ul>
    </section>
  </div>
</template>
