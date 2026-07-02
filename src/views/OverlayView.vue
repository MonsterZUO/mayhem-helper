<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentChampionId, useMayhemChampion } from '@/composables/mayhem/useMayhemData'
import MayhemBuild from '@/components/features/mayhem/MayhemBuild.vue'

// 浮层：轮询当前对局英雄 → 自动显示其海克斯优先级（数据驱动，不替你三选一）
const championId = ref<number | null>(null)
const { data, isLoading } = useMayhemChampion(championId)

let timer: ReturnType<typeof setInterval> | undefined
async function poll() {
  try {
    championId.value = await getCurrentChampionId()
  } catch {
    // 未连客户端 / 不在选英雄：忽略
  }
}

onMounted(() => {
  poll()
  timer = setInterval(poll, 4000)
})
onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>

<template>
  <div class="h-screen w-screen overflow-y-auto bg-black/80 p-[12px] text-white backdrop-blur-sm">
    <div v-if="championId && data">
      <MayhemBuild :data="data" compact />
    </div>
    <div v-else-if="isLoading" class="pt-[40px] text-center text-[12px] text-white/60">加载中…</div>
    <div v-else class="pt-[40px] text-center text-[12px] leading-[18px] text-white/60">
      进入海克斯大乱斗选英雄后<br />自动显示该英雄的海克斯优先级<br />
      <span class="text-white/40">（游戏需设为无边框模式）</span>
    </div>
  </div>
</template>
