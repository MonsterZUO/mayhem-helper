<script setup lang="ts">
import { computed, ref } from 'vue'
import { ChevronDown, ChevronRight, MoveRight } from 'lucide-vue-next'
import {
  groupAugmentsByRarity,
  itemIconUrl,
  buildRoute,
  useItemNames,
  type MayhemChampion,
  type RankedItem
} from '@/composables/mayhem/useMayhemData'

const props = defineProps<{
  data: MayhemChampion
  /** 紧凑模式（浮层用），减小间距与条目数。 */
  compact?: boolean
  /** 隐藏来源/版本角标（外部已展示时用，避免重复）。 */
  hideMeta?: boolean
}>()

const groups = computed(() => groupAugmentsByRarity(props.data.augments))
const route = computed(() => buildRoute(props.data.core_items))
const topTrios = computed(() => props.data.trios.slice(0, props.compact ? 3 : 5))
const { data: itemNames } = useItemNames()

// 海克斯每档默认只展示 top N，展开看全部（罗列过长不可用）
const PREVIEW_COUNT = computed(() => (props.compact ? 3 : 4))
const expanded = ref<Record<string, boolean>>({})
function toggleExpand(rarity: string) {
  expanded.value[rarity] = !expanded.value[rarity]
}

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
const rarityText: Record<string, string> = {
  棱彩: 'text-[#a21caf] dark:text-[#e04ba0]',
  黄金: 'text-[#a16207] dark:text-[#e0a72c]',
  白银: 'text-[#64748b] dark:text-[#9aa4b2]',
  未知: 'text-[#6b7280]'
}
const rarityBar: Record<string, string> = {
  棱彩: 'bg-[#a21caf]/50 dark:bg-[#e04ba0]/50',
  黄金: 'bg-[#a16207]/50 dark:bg-[#e0a72c]/50',
  白银: 'bg-[#64748b]/50 dark:bg-[#9aa4b2]/50',
  未知: 'bg-[#6b7280]/50'
}

/** 胜率条宽度：40%~70% 映射 0~100%，差异放大可辨。 */
function barWidth(winRate: number): string {
  const w = Math.max(0, Math.min(1, (winRate - 0.4) / 0.3))
  return `${Math.round(w * 100)}%`
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
      <div class="flex flex-wrap items-center gap-x-[18px] gap-y-[10px] rounded-[10px] bg-accent/30 px-[12px] py-[10px]">
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
          <img :src="itemIconUrl(it.id)" :alt="itemName(it.id)" class="h-[26px] w-[26px] rounded-[5px] border border-border/40" loading="lazy" />
          <span class="flex-1 truncate text-[12px] text-foreground/85">{{ itemName(it.id) }}</span>
          <span class="text-[12px] font-[500] tabular-nums text-foreground/80">{{ pct(it.win_rate) }}</span>
        </li>
      </ul>
    </section>

    <!-- ═══ 海克斯优先级：每档 top N + 展开，胜率条 ═══ -->
    <section v-for="group in groups" :key="group.rarity" class="flex flex-col gap-[5px]">
      <div class="flex items-center gap-[6px]">
        <span class="h-[8px] w-[8px] rounded-full" :class="rarityDot[group.label]" />
        <span class="text-[13px] font-[600]" :class="rarityText[group.label]">{{ group.label }}海克斯</span>
        <span class="text-[11px] text-muted-foreground">{{ group.items.length }} 个</span>
      </div>
      <ul class="flex flex-col gap-[3px]">
        <li
          v-for="aug in expanded[group.rarity] ? group.items : group.items.slice(0, PREVIEW_COUNT)"
          :key="aug.id"
          class="relative flex items-center gap-[8px] overflow-hidden rounded-[7px] bg-accent/25 px-[8px] py-[4px]"
        >
          <!-- 胜率条背景 -->
          <div
            class="absolute inset-y-0 left-0 opacity-25"
            :class="rarityBar[group.label]"
            :style="{ width: barWidth(aug.win_rate) }"
          />
          <img
            v-if="aug.icon_url"
            :src="aug.icon_url"
            :alt="aug.name"
            class="relative h-[22px] w-[22px] rounded-[4px]"
            loading="lazy"
          />
          <span class="relative flex-1 truncate text-[12.5px] text-foreground/90">{{ aug.name }}</span>
          <span class="relative text-[12px] font-[500] tabular-nums text-foreground/85">{{ pct(aug.win_rate) }}</span>
        </li>
      </ul>
      <button
        v-if="group.items.length > PREVIEW_COUNT"
        class="flex items-center gap-[3px] self-start rounded px-[6px] py-[2px] text-[11px] text-muted-foreground hover:bg-accent/40 hover:text-foreground"
        @click="toggleExpand(group.rarity)"
      >
        <component :is="expanded[group.rarity] ? ChevronDown : ChevronRight" class="h-[12px] w-[12px]" />
        {{ expanded[group.rarity] ? '收起' : `展开全部 ${group.items.length} 个` }}
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
          >T{{ trio.win_rate_tier }}</span>
          <span class="flex-1 truncate text-[12.5px] text-foreground/85">{{ trio.names.join('  +  ') }}</span>
          <span class="text-[11px] tabular-nums text-muted-foreground">{{ trio.num_games }} 局</span>
        </li>
      </ul>
    </section>
  </div>
</template>
