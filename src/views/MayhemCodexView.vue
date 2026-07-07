<script setup lang="ts">
import { ref, computed } from 'vue'
import { toast } from 'vue-sonner'
import { Dices, Search, ExternalLink, PanelTopOpen, Loader2, Sparkles, ArrowLeft } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import ChampionSelector from '@/components/features/auto-function/ChampionSelector.vue'
import MayhemBuild from '@/components/features/mayhem/MayhemBuild.vue'
import { getChampionIconUrlByAlias } from '@/lib'
import { useChampionSummaryQuery } from '@/composables/useLolApiQuery'
import {
  useMayhemChampion,
  useMayhemAugmentTiers,
  useMayhemAugmentDetail,
  applyMayhemItemSet,
  toggleOverlay,
  rarityLabel
} from '@/composables/mayhem/useMayhemData'

interface ChampionSummary {
  id: number
  name: string
  alias?: string
  squarePortraitPath?: string
}

const selected = ref<ChampionSummary | null>(null)
const importing = ref(false)
const importingRunes = ref(false)
const tab = ref<'champion' | 'tiers'>('champion')

const championId = computed(() => selected.value?.id ?? null)
const { data, isLoading, isError } = useMayhemChampion(championId)

const tiersEnabled = computed(() => tab.value === 'tiers' && !selected.value)
const { data: tiers, isLoading: tiersLoading } = useMayhemAugmentTiers(tiersEnabled)

// 海克斯详情（反向索引：全局榜点击进入）
const selectedAugmentId = ref<number | null>(null)
const { data: augmentDetail, isLoading: augmentDetailLoading } = useMayhemAugmentDetail(selectedAugmentId)

// champion_id → 名字/别名（cdragon champion-summary，详情榜显示英雄用）
const { data: championSummaries } = useChampionSummaryQuery()
const championById = computed(() => {
  const map = new Map<number, { name: string; alias: string }>()
  for (const c of championSummaries.value ?? []) {
    map.set(c.id, { name: c.name, alias: c.alias })
  }
  return map
})

// 稀有度色分明暗两套（同 MayhemBuild）
const rarityText: Record<string, string> = {
  prismatic: 'text-[#a21caf] dark:text-[#e04ba0]',
  gold: 'text-[#a16207] dark:text-[#e0a72c]',
  silver: 'text-[#64748b] dark:text-[#9aa4b2]',
  unknown: 'text-[#6b7280]'
}

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

async function importRunes() {
  if (!selected.value?.alias) return
  importingRunes.value = true
  try {
    // 复用既有符文导入（op.gg 数据源，取首个推荐符文页写入客户端）
    const msg = await invoke<string>('apply_champion_build', {
      championAlias: selected.value.alias,
      buildIndex: 0
    })
    toast.success(msg)
  } catch (err) {
    toast.error(`符文导入失败: ${err}`)
  } finally {
    importingRunes.value = false
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
        <p class="text-[12px] leading-[16px] text-muted-foreground">选英雄看海克斯优先级 · 一键导入出装 · 游戏内浮层</p>
      </div>
    </div>

    <!-- 未选英雄：tab 切换 选英雄 / 全局榜 -->
    <div v-if="!selected" class="rounded-[14px] border border-border/60 bg-card/40 p-[20px]">
      <div class="mb-[14px] flex items-center gap-[8px]">
        <button
          class="h-[32px] rounded-[8px] px-[14px] text-[13px] font-[500] transition"
          :class="tab === 'champion' ? 'bg-primary text-primary-foreground' : 'text-foreground/70 hover:bg-accent'"
          @click="tab = 'champion'"
        >
          查英雄
        </button>
        <button
          class="h-[32px] rounded-[8px] px-[14px] text-[13px] font-[500] transition"
          :class="tab === 'tiers' ? 'bg-primary text-primary-foreground' : 'text-foreground/70 hover:bg-accent'"
          @click="tab = 'tiers'"
        >
          全局海克斯榜
        </button>
      </div>

      <template v-if="tab === 'champion'">
        <div class="mb-[14px] flex items-center gap-[8px] text-[13px] text-muted-foreground">
          <Search class="h-[15px] w-[15px]" />
          搜索并选择一个英雄，查看它的海克斯大乱斗推荐
        </div>
        <ChampionSelector @select="onSelect" />
      </template>

      <template v-else>
        <!-- 海克斯详情：反向索引（该海克斯适配哪些英雄） -->
        <template v-if="selectedAugmentId">
          <button
            class="mb-[12px] flex h-[30px] items-center gap-[6px] rounded-[8px] border border-border/60 px-[12px] text-[13px] text-foreground/80 transition hover:bg-accent"
            @click="selectedAugmentId = null"
          >
            <ArrowLeft class="h-[14px] w-[14px]" /> 返回全局榜
          </button>
          <div
            v-if="augmentDetailLoading"
            class="flex items-center gap-[8px] py-[40px] justify-center text-[13px] text-muted-foreground"
          >
            <Loader2 class="h-[15px] w-[15px] animate-spin" /> 聚合该海克斯分英雄数据中…
          </div>
          <div v-else-if="augmentDetail" class="flex flex-col gap-[12px]">
            <div class="flex items-center gap-[12px]">
              <img
                v-if="augmentDetail.icon_url"
                :src="augmentDetail.icon_url"
                :alt="augmentDetail.name"
                class="h-[40px] w-[40px] rounded-[8px]"
              />
              <div class="flex-1">
                <div class="flex items-center gap-[8px]">
                  <span class="text-[16px] font-[600] text-foreground">{{ augmentDetail.name }}</span>
                  <span class="text-[12px]" :class="rarityText[augmentDetail.rarity]">{{
                    rarityLabel(augmentDetail.rarity)
                  }}</span>
                </div>
                <div class="mt-[2px] text-[11px] text-muted-foreground">
                  适配英雄排行（按胜率） · {{ augmentDetail.source }} · 版本 {{ augmentDetail.patch }}
                </div>
              </div>
            </div>
            <ul v-if="augmentDetail.champions.length" class="flex flex-col gap-[4px]">
              <li
                v-for="(entry, i) in augmentDetail.champions.slice(0, 40)"
                :key="entry.champion_id"
                class="flex items-center gap-[10px] rounded-[8px] border border-border/40 px-[10px] py-[6px]"
              >
                <span class="w-[26px] text-right text-[12px] tabular-nums text-muted-foreground">{{ i + 1 }}</span>
                <img
                  v-if="championById.get(entry.champion_id)?.alias"
                  :src="getChampionIconUrlByAlias(championById.get(entry.champion_id)!.alias)"
                  :alt="championById.get(entry.champion_id)?.name"
                  class="h-[24px] w-[24px] rounded-[4px]"
                  loading="lazy"
                />
                <span class="flex-1 truncate text-[13px] text-foreground/90">
                  {{ championById.get(entry.champion_id)?.name ?? `英雄 ${entry.champion_id}` }}
                </span>
                <span class="w-[56px] text-right text-[13px] font-[500] tabular-nums text-foreground/90">{{
                  pct(entry.win_rate)
                }}</span>
                <span class="w-[88px] text-right text-[11px] tabular-nums text-muted-foreground"
                  >{{ entry.num_games.toLocaleString() }} 局</span
                >
              </li>
            </ul>
            <div v-else class="py-[24px] text-center text-[13px] text-muted-foreground">暂无达到样本门槛的英雄数据</div>
            <div class="text-[11px] text-muted-foreground">
              单英雄样本 ≥{{ augmentDetail.min_games }} 局才入榜<template v-if="augmentDetail.filtered_out">
                · 另有 {{ augmentDetail.filtered_out }} 个英雄样本不足未入榜</template
              >
            </div>
          </div>
        </template>

        <!-- 全局榜 -->
        <template v-else>
          <div
            v-if="tiersLoading"
            class="flex items-center gap-[8px] py-[40px] justify-center text-[13px] text-muted-foreground"
          >
            <Loader2 class="h-[15px] w-[15px] animate-spin" /> 聚合 173 英雄数据中…
          </div>
          <div v-else-if="tiers" class="flex flex-col gap-[8px]">
            <div class="flex items-center justify-between text-[11px] text-muted-foreground">
              <span>按对局数加权的全服胜率 · {{ tiers.source }} · 版本 {{ tiers.patch }} · 点击查看适配英雄</span>
              <span>{{ tiers.augments.length }} 个海克斯（≥500 局）</span>
            </div>
            <ul class="flex flex-col gap-[4px]">
              <li
                v-for="(aug, i) in tiers.augments.slice(0, 40)"
                :key="aug.id"
                class="flex cursor-pointer items-center gap-[10px] rounded-[8px] border border-border/40 px-[10px] py-[6px] transition hover:bg-accent"
                @click="selectedAugmentId = aug.id"
              >
                <span class="w-[26px] text-right text-[12px] tabular-nums text-muted-foreground">{{ i + 1 }}</span>
                <img
                  v-if="aug.icon_url"
                  :src="aug.icon_url"
                  :alt="aug.name"
                  class="h-[24px] w-[24px] rounded-[4px]"
                  loading="lazy"
                />
                <span class="flex-1 truncate text-[13px] text-foreground/90">{{ aug.name }}</span>
                <span class="w-[42px] text-[11px]" :class="rarityText[aug.rarity]">{{ rarityLabel(aug.rarity) }}</span>
                <span class="w-[56px] text-right text-[13px] font-[500] tabular-nums text-foreground/90">{{
                  pct(aug.win_rate)
                }}</span>
                <span class="w-[88px] text-right text-[11px] tabular-nums text-muted-foreground"
                  >{{ aug.num_games.toLocaleString() }} 局</span
                >
              </li>
            </ul>
          </div>
        </template>
      </template>
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
            <span v-if="data" class="rounded-[6px] bg-primary/12 px-[8px] py-[2px] text-[12px] font-[500] text-primary"
              >胜率 {{ pct(data.win_rate) }}</span
            >
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
            :disabled="importingRunes"
            class="flex h-[36px] items-center gap-[6px] rounded-[9px] border border-primary/40 px-[14px] text-[13px] font-[500] text-primary transition hover:bg-primary/10 disabled:opacity-50"
            @click="importRunes"
          >
            <Loader2 v-if="importingRunes" class="h-[15px] w-[15px] animate-spin" />
            <Sparkles v-else class="h-[15px] w-[15px]" />
            {{ importingRunes ? '导入中' : '导入符文' }}
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
      <div
        v-if="isLoading"
        class="flex items-center justify-center gap-[8px] py-[60px] text-[14px] text-muted-foreground"
      >
        <Loader2 class="h-[16px] w-[16px] animate-spin" /> 加载推荐中…
      </div>
      <div
        v-else-if="isError"
        class="rounded-[12px] border border-destructive/30 bg-destructive/5 p-[16px] text-[13px] text-destructive"
      >
        取数失败。国服网络下外服数据源可能连不上，会自动回退出厂快照——若持续失败请检查网络。
      </div>
      <div v-else-if="data" class="rounded-[14px] border border-border/60 bg-card/40 p-[18px]">
        <MayhemBuild :data="data" hide-meta />
      </div>
    </template>
  </div>
</template>
