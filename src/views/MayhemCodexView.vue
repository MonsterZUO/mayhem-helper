<script setup lang="ts">
import { ref, computed } from 'vue'
import { toast } from 'vue-sonner'
import { Dices, Search, ExternalLink, PanelTopOpen, Loader2 } from 'lucide-vue-next'
import ChampionSelector from '@/components/features/auto-function/ChampionSelector.vue'
import MayhemBuild from '@/components/features/mayhem/MayhemBuild.vue'
import { getChampionIconUrlByAlias } from '@/lib'
import {
  useMayhemChampion,
  applyMayhemItemSet,
  toggleOverlay
} from '@/composables/mayhem/useMayhemData'

interface ChampionSummary {
  id: number
  name: string
  alias?: string
  squarePortraitPath?: string
}

const selected = ref<ChampionSummary | null>(null)
const importing = ref(false)

const championId = computed(() => selected.value?.id ?? null)
const { data, isLoading, isError } = useMayhemChampion(championId)

function portrait(c: ChampionSummary): string {
  return c.alias ? getChampionIconUrlByAlias(c.alias) : ''
}

function onSelect(c: ChampionSummary) {
  selected.value = c
}

function reset() {
  selected.value = null
}

async function importItemSet() {
  if (!championId.value || !data.value) return
  const itemIds = data.value.core_items.slice(0, 6).map((i) => i.id)
  if (itemIds.length === 0) {
    toast.error('该英雄暂无核心出装数据')
    return
  }
  importing.value = true
  try {
    toast.success(await applyMayhemItemSet(championId.value, itemIds))
  } catch (err) {
    toast.error(`导入失败: ${err}`)
  } finally {
    importing.value = false
  }
}

async function openOverlay() {
  try {
    await toggleOverlay()
    toast.success('已切换游戏内浮层')
  } catch (err) {
    toast.error(`浮层切换失败: ${err}`)
  }
}

function pct(v: number): string {
  return `${(v * 100).toFixed(1)}%`
}
</script>

<template>
  <div class="flex flex-col gap-[16px]">
    <!-- 标题条 -->
    <div class="flex items-center gap-[12px]">
      <div class="flex h-[40px] w-[40px] items-center justify-center rounded-[10px] bg-primary/12 text-primary">
        <Dices class="h-[22px] w-[22px]" />
      </div>
      <div>
        <h1 class="text-[18px] font-[600] leading-[22px] text-foreground">海克斯大乱斗</h1>
        <p class="text-[12px] leading-[16px] text-muted-foreground">
          选英雄看海克斯优先级 · 一键导入出装 · 游戏内浮层
        </p>
      </div>
    </div>

    <!-- 未选英雄：英雄选择器 -->
    <div v-if="!selected" class="rounded-[14px] border border-border/60 bg-card/40 p-[20px]">
      <div class="mb-[14px] flex items-center gap-[8px] text-[13px] text-muted-foreground">
        <Search class="h-[15px] w-[15px]" />
        搜索并选择一个英雄，查看它的海克斯大乱斗推荐
      </div>
      <ChampionSelector @select="onSelect" />
    </div>

    <!-- 已选英雄 -->
    <template v-else>
      <!-- 英雄头部卡 -->
      <div class="flex items-center gap-[14px] rounded-[14px] border border-border/60 bg-card/50 p-[14px]">
        <img
          v-if="portrait(selected)"
          :src="portrait(selected)"
          :alt="selected.name"
          class="h-[56px] w-[56px] rounded-[12px] border border-border/50 object-cover"
        />
        <div class="flex-1">
          <div class="flex items-center gap-[10px]">
            <span class="text-[18px] font-[600] text-foreground">{{ selected.name }}</span>
            <span
              v-if="data"
              class="rounded-[6px] bg-primary/12 px-[8px] py-[2px] text-[12px] font-[500] text-primary"
            >胜率 {{ pct(data.win_rate) }}</span>
            <span v-if="data" class="text-[12px] text-muted-foreground">登场 {{ pct(data.pick_rate) }}</span>
          </div>
          <div class="mt-[3px] text-[12px] text-muted-foreground">
            <span v-if="data">数据来源 {{ data.source }} · 版本 {{ data.patch }}</span>
            <span v-else>加载中…</span>
          </div>
        </div>
        <div class="flex items-center gap-[8px]">
          <button
            :disabled="importing || !data"
            class="flex h-[36px] items-center gap-[6px] rounded-[9px] bg-primary px-[14px] text-[13px] font-[500] text-primary-foreground transition hover:opacity-90 disabled:opacity-50"
            @click="importItemSet"
          >
            <Loader2 v-if="importing" class="h-[15px] w-[15px] animate-spin" />
            <ExternalLink v-else class="h-[15px] w-[15px]" />
            {{ importing ? '导入中' : '导入出装' }}
          </button>
          <button
            class="flex h-[36px] items-center gap-[6px] rounded-[9px] border border-border/60 px-[14px] text-[13px] text-foreground/80 transition hover:bg-accent"
            @click="openOverlay"
          >
            <PanelTopOpen class="h-[15px] w-[15px]" />
            浮层
          </button>
          <button
            class="h-[36px] rounded-[9px] border border-border/60 px-[14px] text-[13px] text-foreground/70 transition hover:bg-accent"
            @click="reset"
          >
            换英雄
          </button>
        </div>
      </div>

      <!-- 出装/海克斯/套路 -->
      <div v-if="isLoading" class="flex items-center justify-center gap-[8px] py-[60px] text-[14px] text-muted-foreground">
        <Loader2 class="h-[16px] w-[16px] animate-spin" /> 加载推荐中…
      </div>
      <div v-else-if="isError" class="rounded-[12px] border border-destructive/30 bg-destructive/5 p-[16px] text-[13px] text-destructive">
        取数失败。国服网络下外服数据源可能连不上，会自动回退出厂快照——若持续失败请检查网络。
      </div>
      <div v-else-if="data" class="rounded-[14px] border border-border/60 bg-card/40 p-[18px]">
        <MayhemBuild :data="data" hide-meta />
      </div>
    </template>
  </div>
</template>
