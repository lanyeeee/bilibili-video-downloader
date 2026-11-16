<script setup lang="ts">
import { onMounted, ref, useTemplateRef } from 'vue'
import { commands, events } from '../../bindings.ts'
import { useStore } from '../../store.ts'
import UncompletedProgresses from './components/UncompletedProgresses.vue'
import CompletedProgresses from './components/CompletedProgresses.vue'
import DownloadDirInput from './components/DownloadDirInput.vue'
import { PhCheckCircle, PhCloudArrowDown } from '@phosphor-icons/vue'
import { NBadge, NPagination, NTabPane, NTabs } from 'naive-ui'
import { createProgressData, createProgressUpdate, getStateIndicator } from './progress.ts'

const store = useStore()

const currentTabName = ref<'uncompleted' | 'completed'>('uncompleted')

const uncompletedProgressesRef = useTemplateRef<InstanceType<typeof UncompletedProgresses>>('uncompletedProgressesRef')
const completedProgressesRef = useTemplateRef<InstanceType<typeof CompletedProgresses>>('completedProgressesRef')

onMounted(async () => {
  await events.downloadEvent.listen(({ payload: { event, data } }) => {
    if (event === 'Speed') {
      store.downloadSpeed = data.speed
    } else if (event === 'TaskCreate') {
      const { progress, state } = data
      store.setProgress(createProgressData(progress, state))
    } else if (event === 'TaskStateUpdate') {
      const { task_id, state } = data

      store.patchProgress(task_id, { state, stateIndicator: getStateIndicator(state) })
    } else if (event === 'TaskSleeping') {
      const { task_id, remaining_sec } = data
      store.patchProgress(task_id, { taskIndicator: `将在${remaining_sec}秒后继续下载` })
    } else if (event === 'TaskDelete') {
      const { task_id } = data
      store.deleteProgress(task_id)
    } else if (event === 'ProgressPreparing') {
      const { task_id } = data
      store.patchProgress(task_id, { taskIndicator: '正在准备下载' })
    } else if (event === 'ProgressUpdate') {
      const progress = data.progress
      store.patchProgress(progress.task_id, (current) => createProgressUpdate(current, progress))
    }
  })

  const result = await commands.restoreDownloadTasks()
  if (result.status === 'error') {
    console.error(result.error)
    return
  }

  for (const { state, progress } of result.data) {
    store.setProgress(createProgressData(progress, state))
  }
})
</script>

<template>
  <div class="h-full flex flex-col overflow-auto" v-if="store.config !== undefined">
    <div class="flex items-center m-2 mb-0">
      <DownloadDirInput />
      <span class="ml-2 whitespace-nowrap">下载速度：{{ store.downloadSpeed }}</span>
    </div>
    <n-tabs
      class="h-full overflow-auto"
      type="line"
      v-model:value="currentTabName"
      animated
      size="small"
      placement="bottom">
      <n-tab-pane class="h-full p-0! overflow-auto flex flex-col" name="uncompleted">
        <template #tab>
          <n-badge :value="store.uncompletedProgressesCount" :offset="[5, -3]" class="tab-badge-wrapper">
            <PhCloudArrowDown :weight="currentTabName === 'uncompleted' ? 'fill' : 'regular'" size="20" />
            <span class="ml-1">下载中</span>
          </n-badge>
        </template>
        <UncompletedProgresses ref="uncompletedProgressesRef" />
      </n-tab-pane>

      <n-tab-pane class="h-full p-0! overflow-auto flex flex-col" name="completed">
        <template #tab>
          <PhCheckCircle :weight="currentTabName === 'completed' ? 'fill' : 'regular'" size="20" />
          <span class="ml-1">已完成</span>
        </template>

        <CompletedProgresses ref="completedProgressesRef" />
      </n-tab-pane>

      <template #suffix>
        <n-pagination
          v-if="currentTabName === 'uncompleted' && uncompletedProgressesRef !== null"
          class="ml-auto mr-2"
          :page-count="uncompletedProgressesRef.pageCount"
          v-model:page="uncompletedProgressesRef.currentPage" />

        <n-pagination
          v-else-if="currentTabName === 'completed' && completedProgressesRef !== null"
          class="ml-auto mr-2"
          :page-count="completedProgressesRef.pageCount"
          v-model:page="completedProgressesRef.currentPage" />
      </template>
    </n-tabs>
  </div>
</template>
<style scoped>
:deep(.n-tabs-nav-scroll-wrapper) {
  @apply mt-2 h-9;
}

:deep(.n-tabs-nav__suffix) {
  @apply important-border-0;
}

:deep(.n-tabs-nav-scroll-wrapper) {
  @apply overflow-visible;
}

:deep(.v-x-scroll) {
  @apply overflow-visible;
}
</style>
