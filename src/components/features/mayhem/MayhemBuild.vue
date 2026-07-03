<script setup lang="ts">
import { computed } from 'vue'
import {
  groupAugmentsByRarity,
  itemIconUrl,
  type MayhemChampion
} from '@/composables/mayhem/useMayhemData'

const props = defineProps<{
  data: MayhemChampion
  /** 紧凑模式（浮层用），减小间距。 */
  compact?: boolean
  /** 隐藏来源/版本角标（外部已展示时用，避免重复）。 */
  hideMeta?: boolean
}>()

const groups = computed(() => groupAugmentsByRarity(props.data.augments))
const topTrios = computed(() => props.data.trios.slice(0, props.compact ? 3 : 8))
const coreItems = computed(() => props.data.core_items.slice(0, props.compact ? 6 : 12))

function pct(v: number): string {
  return `${(v * 100).toFixed(1)}%`
}

const rarityClass: Record<string, string> = {
  棱彩: 'text-[#e04ba0] border-[#e04ba0]/40',
  黄金: 'text-[#e0a72c] border-[#e0a72c]/40',
  白银: 'text-[#9aa4b2] border-[#9aa4b2]/40',
  未知: 'text-[#6b7280] border-[#6b7280]/40'
}
</script>

<template>
  <div class="flex flex-col" :class="compact ? 'gap-[10px]' : 'gap-[18px]'">
    <!-- 来源 / 版本 角标（数据地区代理透明标注，见 ADR-0001） -->
    <div v-if="!hideMeta" class="flex items-center justify-between text-[11px] text-foreground/50">
      <span>数据来源 {{ data.source }} · 版本 {{ data.patch }}</span>
      <span>胜率 {{ pct(data.win_rate) }} · 登场 {{ pct(data.pick_rate) }}</span>
    </div>

    <!-- 海克斯优先级：按稀有度分组、组内按胜率降序 -->
    <section
      v-for="group in groups"
      :key="group.rarity"
      class="flex flex-col gap-[6px]"
    >
      <div class="text-[12px] font-[600]" :class="rarityClass[group.label]">
        {{ group.label }}海克斯
      </div>
      <ul class="flex flex-col gap-[4px]">
        <li
          v-for="aug in group.items"
          :key="aug.id"
          class="flex items-center gap-[8px] rounded-[6px] border px-[8px] py-[5px]"
          :class="rarityClass[group.label]"
        >
          <img
            v-if="aug.icon_url"
            :src="aug.icon_url"
            :alt="aug.name"
            class="h-[22px] w-[22px] rounded-[4px] object-cover"
            loading="lazy"
          />
          <span class="flex-1 truncate text-[13px] text-foreground/90">{{ aug.name }}</span>
          <span class="text-[12px] text-foreground/70">{{ pct(aug.win_rate) }}</span>
        </li>
      </ul>
    </section>

    <!-- 最优三连组合 -->
    <section v-if="topTrios.length" class="flex flex-col gap-[6px]">
      <div class="text-[12px] font-[600] text-foreground/70">最优三连套路</div>
      <ul class="flex flex-col gap-[4px]">
        <li
          v-for="(trio, i) in topTrios"
          :key="i"
          class="flex items-center gap-[6px] rounded-[6px] bg-accent/40 px-[8px] py-[5px]"
        >
          <span class="flex-1 truncate text-[12px] text-foreground/85">
            {{ trio.names.join(' + ') }}
          </span>
          <span class="rounded-[4px] bg-primary/15 px-[6px] text-[11px] text-primary">
            T{{ trio.win_rate_tier }}
          </span>
        </li>
      </ul>
    </section>

    <!-- 核心出装 -->
    <section v-if="coreItems.length" class="flex flex-col gap-[6px]">
      <div class="text-[12px] font-[600] text-foreground/70">核心出装</div>
      <div class="flex flex-wrap gap-[6px]">
        <div
          v-for="item in coreItems"
          :key="item.id"
          class="relative"
          :title="`胜率 ${pct(item.win_rate)}`"
        >
          <img
            :src="itemIconUrl(item.id)"
            :alt="String(item.id)"
            class="h-[32px] w-[32px] rounded-[4px] border border-border/40 object-cover"
            loading="lazy"
          />
        </div>
      </div>
    </section>
  </div>
</template>
