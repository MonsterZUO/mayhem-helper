<script setup lang="ts">
import { ref } from 'vue'
import { toast } from 'vue-sonner'
import {
  useMayhemChampion,
  applyMayhemItemSet,
  toggleOverlay
} from '@/composables/mayhem/useMayhemData'
import MayhemBuild from '@/components/features/mayhem/MayhemBuild.vue'

// 速查库 / 出装助手：查任意英雄的海克斯大乱斗推荐 + 一键导入出装
const input = ref<number | null>(null)
const championId = ref<number | null>(null)
const importing = ref(false)

const { data, isLoading, isError } = useMayhemChampion(championId)

function search() {
  if (input.value && input.value > 0) {
    championId.value = input.value
  }
}

async function importItemSet() {
  if (!championId.value || !data.value) return
  importing.value = true
  try {
    const itemIds = data.value.core_items.slice(0, 6).map((i) => i.id)
    const msg = await applyMayhemItemSet(championId.value, itemIds)
    toast.success(msg)
  } catch (err) {
    toast.error(`导入失败: ${err}`)
  } finally {
    importing.value = false
  }
}

async function openOverlay() {
  try {
    await toggleOverlay()
  } catch (err) {
    toast.error(`浮层切换失败: ${err}`)
  }
}
</script>

<template>
  <div class="flex flex-col gap-[16px]">
    <div class="flex items-center gap-[10px]">
      <input
        v-model.number="input"
        type="number"
        placeholder="英雄 ID（如 5 = 德邦）"
        class="h-[36px] w-[220px] rounded-[8px] border border-border/50 bg-background px-[12px] text-[14px] outline-none focus:border-primary"
        @keyup.enter="search"
      />
      <button
        class="h-[36px] rounded-[8px] bg-primary px-[16px] text-[14px] font-[500] text-primary-foreground hover:opacity-90"
        @click="search"
      >
        查询
      </button>
      <button
        class="h-[36px] rounded-[8px] border border-border/50 px-[16px] text-[14px] hover:bg-accent"
        @click="openOverlay"
      >
        切换游戏内浮层
      </button>
    </div>

    <div v-if="isLoading" class="text-[14px] text-foreground/60">加载中…</div>
    <div v-else-if="isError" class="text-[14px] text-red-400">
      取数失败（Blitz 不可达？国服网络下该外服端点可能连不上，见 README）
    </div>
    <div v-else-if="data" class="flex flex-col gap-[14px]">
      <div class="flex items-center gap-[10px]">
        <button
          :disabled="importing"
          class="h-[34px] rounded-[8px] bg-primary/90 px-[14px] text-[13px] font-[500] text-primary-foreground hover:opacity-90 disabled:opacity-50"
          @click="importItemSet"
        >
          {{ importing ? '导入中…' : '一键导入出装到客户端' }}
        </button>
      </div>
      <MayhemBuild :data="data" />
    </div>
    <div v-else class="text-[14px] text-foreground/50">输入英雄 ID 查询海克斯大乱斗出装/海克斯/套路</div>
  </div>
</template>
