<script setup lang="tsx">
import { ref, watchEffect, computed, nextTick, DeepReadonly, watch } from 'vue'
import { SelectionArea, SelectionEvent } from '@viselect/vue'
import { commands } from '../../../bindings.ts'
import { DropdownOption, NIcon } from 'naive-ui'
import { PhChecks, PhTrash, PhArrowClockwise } from '@phosphor-icons/vue'
import { useStore } from '../../../store.ts'
import DownloadProgress from './DownloadProgress.vue'
import { ProgressData } from '../DownloadPane.vue'

const store = useStore()

const selectedIds = ref<Set<string>>(new Set())
const selectionAreaRef = ref<InstanceType<typeof SelectionArea>>()
const progressRefs = ref<InstanceType<typeof DownloadProgress>[]>([])
const { dropdownX, dropdownY, dropdownShowing, dropdownOptions, showDropdown } = useDropdown()

const completedProgresses = computed<[string, DeepReadonly<ProgressData>][]>(() =>
  Array.from(store.progresses.entries())
    .filter(([, { state }]) => state === 'Completed')
    .sort((a, b) => (b[1].completed_ts ?? 0) - (a[1].completed_ts ?? 0)),
)

const PAGE_SIZE = 20
const currentPage = ref<number>(1)
const pageCount = computed<number>(() => {
  return Math.ceil(completedProgresses.value.length / PAGE_SIZE)
})
watchEffect(() => {
  if (currentPage.value > pageCount.value) {
    currentPage.value = Math.max(1, pageCount.value)
  }
})

const currentPageProgresses = computed<[string, DeepReadonly<ProgressData>][]>(() => {
  const start = (currentPage.value - 1) * PAGE_SIZE
  const end = start + PAGE_SIZE
  return completedProgresses.value.slice(start, end)
})

watchEffect(() => {
  // 保证selectedIds中的任务ID都在completedProgresses中
  const completedIds = new Set(completedProgresses.value.map(([taskId]) => taskId))
  for (const taskId of selectedIds.value) {
    if (!completedIds.has(taskId)) {
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
          completedProgresses.value.forEach(([taskId]) => selectedIds.value.add(taskId))
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
          commands.restartDownloadTasks(Array.from(selectedIds.value))
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
          commands.deleteDownloadTasks(Array.from(selectedIds.value).reverse())
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
      <div class="animate-pulse text-violet">左键拖动进行框选，右键打开菜单</div>

      <DownloadProgress
        v-for="[taskId, p] in currentPageProgresses"
        :key="taskId"
        :p="p"
        v-model:selected-ids="selectedIds"
        ref="progressRefs"
        :data-key="taskId"
        :class="['selectable', selectedIds.has(taskId) ? 'selected shadow-md' : 'hover:bg-gray-1']" />
      <n-dropdown
        placement="bottom-start"
        trigger="manual"
        :x="dropdownX"
        :y="dropdownY"
        :options="dropdownOptions"
        :show="dropdownShowing"
        :on-clickoutside="() => (dropdownShowing = false)" />
    </SelectionArea>
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
