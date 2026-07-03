<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getCurrentChampionId, useMayhemChampion } from '@/composables/mayhem/useMayhemData'
import MayhemBuild from '@/components/features/mayhem/MayhemBuild.vue'
import { fetchChampionSummary } from '@/lib/dataApi'
import { getChampionIconUrlByAlias } from '@/lib'

// 浮层：轮询当前对局英雄 → 自动显示其海克斯优先级（数据驱动，不替你三选一）
const championId = ref<number | null>(null)
const { data, isLoading } = useMayhemChampion(championId)

// id→{name,alias} 映射（浮层显示当前英雄名，reroll 后一眼可辨）
const championIndex = ref<Map<number, { name: string; alias: string }>>(new Map())
const current = computed(() =>
  championId.value ? championIndex.value.get(championId.value) : undefined
)

let timer: ReturnType<typeof setInterval> | undefined
async function poll() {
  try {
    championId.value = await getCurrentChampionId()
  } catch {
    // 未连客户端 / 不在选英雄：忽略
  }
}

onMounted(async () => {
  poll()
  timer = setInterval(poll, 4000)
  try {
    const res = await fetchChampionSummary()
    const map = new Map<number, { name: string; alias: string }>()
    for (const c of res.data ?? []) {
      if (c.id > 0) map.set(c.id, { name: c.name, alias: c.alias })
    }
    championIndex.value = map
  } catch {
    // 名字映射失败仅影响标题显示，不阻断决策卡
  }
})
onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="h-screen w-screen overflow-y-auto bg-black/80 p-[12px] text-white backdrop-blur-sm">
    <div v-if="championId && data">
      <!-- 当前英雄头行：让玩家一眼知道浮层对应谁 -->
      <div class="mb-[10px] flex items-center gap-[8px] border-b border-white/15 pb-[8px]">
        <img
          v-if="current?.alias"
          :src="getChampionIconUrlByAlias(current.alias)"
          :alt="current?.name"
          class="h-[28px] w-[28px] rounded-[6px]"
        />
        <span class="text-[15px] font-[600] leading-[20px]">{{ current?.name ?? `英雄 ${championId}` }}</span>
        <span class="ml-auto text-[11px] text-white/50">自动跟随本局</span>
      </div>
      <MayhemBuild :data="data" compact />
    </div>
    <div v-else-if="isLoading" class="pt-[40px] text-center text-[12px] text-white/60">加载中…</div>
    <div v-else class="pt-[40px] text-center text-[12px] leading-[18px] text-white/60">
      进入海克斯大乱斗选英雄后<br />自动显示该英雄的海克斯优先级<br />
      <span class="text-white/40">（游戏需设为无边框模式）</span>
    </div>
  </div>
</template>
