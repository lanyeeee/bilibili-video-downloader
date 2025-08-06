<script setup lang="ts">
import { useStore } from '../../../store.ts'
import { useMessage } from 'naive-ui'

const message = useMessage()
const store = useStore()
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col gap-row-2">
    <div class="flex gap-1">
      <n-tooltip placement="top" trigger="hover">
        <div>最多有多少个任务同时下载</div>
        <template #trigger>
          <n-input-group class="w-40%">
            <n-input-group-label size="small">任务并发</n-input-group-label>
            <n-input-number
              class="w-full"
              v-model:value="store.config.task_concurrency"
              size="small"
              @update:value="message.warning('对任务并发的修改需要重启才能生效')"
              :min="1"
              :parse="(x: string) => Number(x)" />
          </n-input-group>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>每个任务下载完成后休息多久</div>
        <template #trigger>
          <n-input-group class="w-60%">
            <n-input-group-label size="small">任务下载间隔</n-input-group-label>
            <n-input-number
              class="w-full"
              v-model:value="store.config.task_download_interval_sec"
              size="small"
              :min="0"
              :parse="(x: string) => Number(x)" />
            <n-input-group-label size="small">秒</n-input-group-label>
          </n-input-group>
        </template>
      </n-tooltip>
    </div>

    <div class="flex gap-1">
      <n-tooltip placement="top" trigger="hover">
        <div>最多有多少个分片同时下载</div>
        <template #trigger>
          <n-input-group class="w-40%">
            <n-input-group-label size="small">分片并发</n-input-group-label>
            <n-input-number
              class="w-full"
              v-model:value="store.config.chunk_concurrency"
              size="small"
              @update-value="message.warning('对分片并发的修改需要重启才能生效')"
              :min="1"
              :parse="(x: string) => Number(x)" />
          </n-input-group>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>每个分片下载完成后休息多久</div>
        <template #trigger>
          <n-input-group class="w-60%">
            <n-input-group-label size="small">分片下载间隔</n-input-group-label>
            <n-input-number
              class="w-full"
              v-model:value="store.config.chunk_download_interval_sec"
              size="small"
              :min="0"
              :parse="(x: string) => Number(x)" />
            <n-input-group-label size="small">秒</n-input-group-label>
          </n-input-group>
        </template>
      </n-tooltip>
    </div>
  </div>
</template>
