<script setup lang="tsx">
import { ref, watchEffect, computed, nextTick, DeepReadonly, watch } from 'vue'
import { SelectionArea, SelectionEvent } from '@viselect/vue'
import { commands } from '../../../bindings.ts'
import { DropdownOption, NIcon } from 'naive-ui'
import { PhPause, PhChecks, PhTrash, PhCaretRight, PhArrowClockwise } from '@phosphor-icons/vue'
import { useStore } from '../../../store.ts'
import DownloadProgress from './DownloadProgress.vue'
import { ProgressData } from '../DownloadPane.vue'

const store = useStore()

const selectedIds = ref<Set<string>>(new Set())
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const progressRefs = ref<InstanceType<typeof DownloadProgress>[]>([])
const { dropdownX, dropdownY, dropdownShowing, dropdownOptions, showDropdown } = useDropdown()

const uncompletedProgresses = computed<[string, DeepReadonly<ProgressData>][]>(() =>
  Array.from(store.progresses.entries())
    .filter(([, { state }]) => state !== 'Completed')
    .sort((a, b) => {
      // 下载中的任务排在最前面
      if (a[1].state === 'Downloading' && b[1].state !== 'Downloading') {
        return -1
      }
      if (b[1].state === 'Downloading' && a[1].state !== 'Downloading') {
        return 1
      }
      // 其次是排队中的任务
      if (a[1].state === 'Pending' && b[1].state !== 'Pending') {
        return -1
      }
      if (b[1].state === 'Pending' && a[1].state !== 'Pending') {
        return 1
      }
      // 再其次是已开始的任务(即有进度的任务)
      if (a[1].taskIndicator !== '' && b[1].taskIndicator === '') {
        return -1
      }
      if (b[1].taskIndicator !== '' && a[1].taskIndicator === '') {
        return 1
      }
      // 如果任务都已开始(即有进度的任务)，则按进度排序
      if (a[1].taskIndicator !== '' && b[1].taskIndicator !== '') {
        return b[1].percentage - a[1].percentage
      }
      // 上述条件都不满足时，按创建时间排序
      return b[1].create_ts - a[1].create_ts
    }),
)

const PAGE_SIZE = 20
const currentPage = ref<number>(1)
const pageCount = computed<number>(() => {
  return Math.ceil(uncompletedProgresses.value.length / PAGE_SIZE)
})
watchEffect(() => {
  if (currentPage.value > pageCount.value) {
    currentPage.value = Math.max(1, pageCount.value)
  }
})

const currentPageProgresses = computed<[string, DeepReadonly<ProgressData>][]>(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE
  const end = start + PAGE_SIZE
  return uncompletedProgresses.value.slice(start, end)
})

watchEffect(() => {
  // 保证selectedIds中的任务ID都在uncompletedProgresses中
  const uncompletedIds = new Set(uncompletedProgresses.value.map(([taskId]) => taskId))
  for (const taskId of selectedIds.value) {
    if (!uncompletedIds.has(taskId)) {
      selectedIds.value.delete(taskId)
    }
  }
})

watch(currentPage, () => {
  selectionAreaRef.value?.$el.scrollTo({ top: 0, behavior: 'instant' })
})

function extractIds(elements: Element[]): string[] {
  return elements
    .map((element) => element.getAttribute('data-key'))
    .filter(Boolean)
    .filter((id) => id !== null)
}

function updateSelectedIds({
  store: {
    changed: { added, removed },
  },
}: SelectionEvent) {
  extractIds(added).forEach((taskId) => selectedIds.value.add(taskId))
  extractIds(removed).forEach((taskId) => selectedIds.value.delete(taskId))
}

function unselectAll({ event, selection }: SelectionEvent) {
  if (!event?.ctrlKey && !event?.metaKey) {
    selection.clearSelection()
    selectedIds.value.clear()
  }
}

function useDropdown() {
  const dropdownX = ref<number>(0)
  const dropdownY = ref<number>(0)
  const dropdownShowing = ref<boolean>(false)
  const dropdownOptions: DropdownOption[] = [
    {
      label: '全选',
      key: 'check all',
      icon: () => (
        <NIcon size="20">
          <PhChecks />
        </NIcon>
      ),
      props: {
        onClick: () => {
          uncompletedProgresses.value.forEach(([taskId]) => selectedIds.value.add(taskId))
          dropdownShowing.value = false
        },
      },
    },
    {
      label: '继续',
      key: 'resume',
      icon: () => (
        <NIcon size="20">
          <PhCaretRight />
        </NIcon>
      ),
      props: {
        onClick: () => {
          commands.resumeDownloadTasks(Array.from(selectedIds.value))
          dropdownShowing.value = false
        },
      },
    },
    {
      label: '暂停',
      key: 'pause',
      icon: () => (
        <NIcon size="20">
          <PhPause />
        </NIcon>
      ),
      props: {
        onClick: () => {
          commands.pauseDownloadTasks(Array.from(selectedIds.value))
          dropdownShowing.value = false
        },
      },
    },
    {
      label: '重来',
      key: 'restart',
      icon: () => (
        <NIcon size="20">
          <PhArrowClockwise />
        </NIcon>
      ),
      props: {
        onClick: () => {
          commands.restartDownloadTasks(Array.from(selectedIds.value).reverse())
          dropdownShowing.value = false
        },
      },
    },
    {
      label: '删除',
      key: 'delete',
      icon: () => (
        <NIcon size="20">
          <PhTrash />
        </NIcon>
      ),
      props: {
        onClick: () => {
          commands.deleteDownloadTasks(Array.from(selectedIds.value))
          dropdownShowing.value = false
        },
      },
    },
  ]

  async function showDropdown(e: MouseEvent) {
    dropdownShowing.value = false
    await nextTick()
    dropdownShowing.value = true
    dropdownX.value = e.clientX
    dropdownY.value = e.clientY
  }

  return {
    dropdownX,
    dropdownY,
    dropdownShowing,
    dropdownOptions,
    showDropdown,
  }
}

defineExpose({
  pageCount,
  currentPage,
})
</script>

<template>
  <div class="h-full flex flex-col overflow-auto">
    <SelectionArea
      ref="selectionAreaRef"
      class="h-full flex flex-col selection-container px-2"
      :options="{ selectables: '.selectable', features: { deselectOnBlur: true } }"
      @contextmenu="showDropdown"
      @move="updateSelectedIds"
      @start="unselectAll">
      <div class="flex">
        <span class="animate-pulse text-violet">
          左键拖动进行框选，右键打开菜单，双击暂停/继续，失败的任务也可以继续
        </span>
      </div>

      <DownloadProgress
        v-for="[taskId, p] in currentPageProgresses"
        :key="taskId"
        :p="p"
        v-model:selected-ids="selectedIds"
        ref="progressRefs"
        :data-key="taskId"
        :class="['selectable ', selectedIds.has(taskId) ? 'selected shadow-md' : 'hover:bg-gray-1']" />
    </SelectionArea>

    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="dropdownX"
      :y="dropdownY"
      :options="dropdownOptions"
      :show="dropdownShowing"
      :on-clickoutside="() => (dropdownShowing = false)" />
  </div>
</template>

<style scoped>
.selection-container {
  @apply select-none overflow-auto;
}

.selection-container .selected {
  @apply bg-[rgb(204,232,255)];
}
</style>
