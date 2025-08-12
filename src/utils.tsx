import { DropdownOption, NIcon } from 'naive-ui'
import { ref, nextTick, render, h } from 'vue'
import { PhChecks, PhCheck, PhX } from '@phosphor-icons/vue'
import { SelectionEvent } from '@viselect/vue'
import TaskToQueueAnimation from './components/TaskToQueueAnimation.vue'
import { EpisodeInfo } from './panes/SearchPane/components/EpisodeCard.vue'

export function extractAid(url: string): number | undefined {
  const parsedUrl = new URL(url)
  const pathname = parsedUrl.pathname
  const segments = pathname.split('/')
  for (const segment of segments) {
    if (segment.toLowerCase().startsWith('av')) {
      const aidString = segment.substring(2)
      const aid = parseInt(aidString, 10)
      if (!isNaN(aid)) {
        return aid
      }
    }
  }
}

export function extractBvid(url: string): string | undefined {
  const parsedUrl = new URL(url)
  const pathname = parsedUrl.pathname
  const segments = pathname.split('/')
  for (const segment of segments) {
    if (segment.toLowerCase().startsWith('bv')) {
      return segment
    }
  }
}

export function extractEpId(url: string): number | undefined {
  const parsedUrl = new URL(url)
  const pathname = parsedUrl.pathname
  const segments = pathname.split('/')
  for (const segment of segments) {
    if (segment.toLowerCase().startsWith('ep')) {
      const epIdString = segment.substring(2)
      const epId = parseInt(epIdString, 10)
      if (!isNaN(epId)) {
        return epId
      }
    }
  }
}

export function extractSeasonId(url: string): number | undefined {
  const parsedUrl = new URL(url)
  const pathname = parsedUrl.pathname
  const segments = pathname.split('/')
  for (const segment of segments) {
    if (segment.toLowerCase().startsWith('ss')) {
      const seasonIdString = segment.substring(2)
      const seasonId = parseInt(seasonIdString, 10)
      if (!isNaN(seasonId)) {
        return seasonId
      }
    }
  }
}

export function extractUid(url: string): number | undefined {
  const parsedUrl = new URL(url)
  if (parsedUrl.hostname === 'space.bilibili.com') {
    const segments = parsedUrl.pathname.split('/')
    const uidSegment = segments[1]
    const uid = parseInt(uidSegment, 10)
    if (!isNaN(uid)) {
      return uid
    }
  }
}

export function extractMediaListId(url: string): number | undefined {
  const parsedUrl = new URL(url)
  const params = new URLSearchParams(parsedUrl.search)
  const fid = params.get('fid')
  if (fid !== null) {
    const mediaListId = parseInt(fid, 10)
    if (!isNaN(mediaListId)) {
      return mediaListId
    }
  }
}

export function useEpisodeDropdown(onCheck: () => void, onUncheck: () => void, onSelectAll: () => void) {
  const dropdownX = ref<number>(0)
  const dropdownY = ref<number>(0)
  const dropdownShowing = ref<boolean>(false)
  const dropdownOptions: DropdownOption[] = [
    {
      label: '勾选',
      key: 'check',
      icon: () => (
        <NIcon size="20">
          <PhCheck />
        </NIcon>
      ),
      props: {
        onClick: onCheck,
      },
    },
    {
      label: '取消勾选',
      key: 'uncheck',
      icon: () => (
        <NIcon size="20">
          <PhX />
        </NIcon>
      ),
      props: {
        onClick: onUncheck,
      },
    },
    {
      label: '全选',
      key: 'select-all',
      icon: () => (
        <NIcon size="20">
          <PhChecks />
        </NIcon>
      ),
      props: {
        onClick: onSelectAll,
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

export function useEpisodeSelection() {
  const selectedIds = ref<Set<number>>(new Set())

  function extractIds(elements: Element[]): number[] {
    return elements
      .map((element) => element.getAttribute('data-key'))
      .filter(Boolean)
      .map(Number)
  }

  function updateSelectedIds({
    store: {
      changed: { added, removed },
    },
  }: SelectionEvent) {
    extractIds(added).forEach((id) => selectedIds.value.add(id))
    extractIds(removed).forEach((id) => selectedIds.value.delete(id))
  }

  function unselectAll({ event, selection }: SelectionEvent) {
    if (!event?.ctrlKey && !event?.metaKey) {
      selection.clearSelection()
      selectedIds.value.clear()
    }
  }

  return {
    selectedIds,
    updateSelectedIds,
    unselectAll,
  }
}

export function playTaskToQueueAnimation(from: Element, to: Element) {
  const startRect = from.getBoundingClientRect()
  const endRect = to.getBoundingClientRect()

  const container = document.createElement('div')
  document.body.appendChild(container)

  const taskToQueueAnimation = h(TaskToQueueAnimation, {
    startX: startRect.left + startRect.width / 2,
    startY: startRect.top + startRect.height / 2,
    endX: endRect.left + endRect.width / 2,
    endY: endRect.top + endRect.height / 2,
    onAnimationEnd: () => {
      render(null, container)
      document.body.removeChild(container)
    },
  })

  render(taskToQueueAnimation, container)
}

// el是否至少有一部分在可视区域内
export function isElementInViewport(el: Element): boolean {
  const rect = el.getBoundingClientRect()
  const viewportHeight = window.innerHeight || document.documentElement.clientHeight
  const viewportWidth = window.innerWidth || document.documentElement.clientWidth

  // 检查垂直方向上是否有交集
  const verticalInView = rect.bottom > 0 && rect.top < viewportHeight

  // 检查水平方向上是否有交集
  const horizontalInView = rect.right > 0 && rect.left < viewportWidth

  // 必须同时在垂直和水平方向上都有交集，才算部分可见
  return verticalInView && horizontalInView
}

export function useEpisodeCard(
  downloadEpisode: (episodeInfo: EpisodeInfo) => Promise<void>,
  checkboxChecked?: (episodeInfo: EpisodeInfo) => boolean,
  handleCheckboxClick?: (episodeInfo: EpisodeInfo) => void,
  handleContextMenu?: (episodeInfo: EpisodeInfo) => void,
) {
  return {
    downloadEpisode,
    checkboxChecked,
    handleCheckboxClick,
    handleContextMenu,
  }
}

export function ensureHttps(url: string): string {
  if (url.startsWith('http://')) {
    return url.replace('http://', 'https://')
  }
  return url
}
