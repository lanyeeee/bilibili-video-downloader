<script setup lang="ts">
import { computed, ref } from 'vue'
import { SelectProps, useMessage } from 'naive-ui'
import { PhMagnifyingGlass } from '@phosphor-icons/vue'
import FloatLabelInput from '../../components/FloatLabelInput.vue'
import { commands, GetBangumiInfoParams, GetNormalInfoParams, SearchParams, SearchResult } from '../../bindings.ts'
import NormalSeasonPanel from './components/NormalSeasonPanel.vue'
import NormalSinglePanel from './components/NormalSinglePanel.vue'
import { extractBvid, extractAid, extractEpId, extractSeasonId } from '../../utils.tsx'
import { useStore } from '../../store.ts'
import BangumiPanel from './components/BangumiPanel.vue'

export type SearchType = 'Auto' | 'Normal' | 'Bangumi'

const searchTypeOptions: SelectProps['options'] = [
  { label: '自动', value: 'Auto' },
  { label: '视频', value: 'Normal' },
  { label: '番剧', value: 'Bangumi' },
]

const store = useStore()

const message = useMessage()

const searchInput = ref<string>('')
const searching = ref<boolean>(false)
const searchTypeSelected = ref<SearchType>('Auto')
const searchResult = ref<SearchResult>()

const searchLabel = computed(() => {
  if (searchTypeSelected.value === 'Normal') {
    return '链接 / av... / BV...'
  } else if (searchTypeSelected.value === 'Bangumi') {
    return '链接 / ep... / ss...'
  }
  return '链接 / av... / BV... / ep... / ss...'
})

async function search(input: string, searchType: SearchType) {
  if (store.currentNavName === undefined) {
    return
  }

  store.currentNavName = 'search'
  searchInput.value = input.trim()
  searchTypeSelected.value = searchType

  const isUrl = input.startsWith('http')

  searching.value = true
  if (searchType === 'Auto') {
    await searchAuto(input, isUrl)
  } else if (searchType === 'Normal') {
    await searchNormal(input, isUrl)
  } else if (searchType === 'Bangumi') {
    await searchBangumi(input, isUrl)
  } else {
    message.error('未知的搜索类型')
  }
  searching.value = false
}

async function searchAuto(input: string, isUrl: boolean) {
  let params: SearchParams | undefined

  if (isUrl) {
    const bvid = extractBvid(input)
    const aid = extractAid(input)
    const epId = extractEpId(input)
    const seasonId = extractSeasonId(input)

    if (bvid !== undefined) {
      params = { Normal: { Bvid: bvid } }
    } else if (aid !== undefined) {
      params = { Normal: { Aid: aid } }
    } else if (epId !== undefined) {
      params = { Bangumi: { EpId: epId } }
    } else if (seasonId !== undefined) {
      params = { Bangumi: { SeasonId: seasonId } }
    }
  } else if (input.toLowerCase().startsWith('bv')) {
    params = { Normal: { Bvid: input } }
  } else if (input.toLowerCase().startsWith('av')) {
    const aid = parseInt(input.substring(2), 10)
    if (!isNaN(aid)) {
      params = { Normal: { Aid: aid } }
    }
  } else if (input.toLowerCase().startsWith('ep')) {
    const epId = parseInt(input.substring(2), 10)
    if (!isNaN(epId)) {
      params = { Bangumi: { EpId: epId } }
    }
  } else if (input.toLowerCase().startsWith('ss')) {
    const seasonId = parseInt(input.substring(2), 10)
    if (!isNaN(seasonId)) {
      params = { Bangumi: { SeasonId: seasonId } }
    }
  }

  if (params === undefined) {
    message.error('解析输入失败，请输入正确的链接或ID(如 av... / BV... / ep... / ss...)')
    return
  }

  const result = await commands.search(params)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  searchResult.value = result.data
}

async function searchNormal(input: string, isUrl: boolean) {
  let params: GetNormalInfoParams | undefined

  if (isUrl) {
    const bvid = extractBvid(input)
    const aid = extractAid(input)
    if (bvid !== undefined) {
      params = { Bvid: bvid }
    } else if (aid !== undefined) {
      params = { Aid: aid }
    }
  } else if (input.toLowerCase().startsWith('bv')) {
    params = { Bvid: input }
  } else if (input.toLowerCase().startsWith('av')) {
    const aid = parseInt(input.substring(2), 10)
    if (!isNaN(aid)) {
      params = { Aid: aid }
    }
  }

  if (params === undefined) {
    message.error('解析输入失败，请输入正确的链接或ID(如 av... / BV...)')
    return
  }

  const result = await commands.search({ Normal: params })
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  searchResult.value = result.data
}

async function searchBangumi(input: string, isUrl: boolean) {
  let params: GetBangumiInfoParams | undefined

  if (isUrl) {
    const epId = extractEpId(input)
    const seasonId = extractSeasonId(input)
    if (epId !== undefined) {
      params = { EpId: epId }
    } else if (seasonId !== undefined) {
      params = { SeasonId: seasonId }
    }
  } else if (input.toLowerCase().startsWith('ep')) {
    const epId = parseInt(input.substring(2), 10)
    if (!isNaN(epId)) {
      params = { EpId: epId }
    }
  } else if (input.toLowerCase().startsWith('ss')) {
    const seasonId = parseInt(input.substring(2), 10)
    if (!isNaN(seasonId)) {
      params = { SeasonId: seasonId }
    }
  }

  if (params === undefined) {
    message.error('解析输入失败，请输入正确的链接或ID(如 ep... / ss...)')
    return
  }

  const result = await commands.search({ Bangumi: params })
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  searchResult.value = result.data
}

defineExpose({ search })
</script>

<template>
  <div class="h-full flex flex-col">
    <n-input-group class="box-border px-2 pt-2">
      <FloatLabelInput
        :label="searchLabel"
        size="small"
        v-model:value="searchInput"
        clearable
        @keydown.enter="search(searchInput.trim(), searchTypeSelected)" />
      <n-select
        class="w-15%"
        v-model:value="searchTypeSelected"
        :options="searchTypeOptions"
        :show-checkmark="false"
        size="small" />
      <n-button
        :loading="searching"
        type="primary"
        size="small"
        class="w-10%"
        @click="search(searchInput.trim(), searchTypeSelected)">
        <template #icon>
          <n-icon size="22">
            <PhMagnifyingGlass weight="bold" />
          </n-icon>
        </template>
      </n-button>
    </n-input-group>
    <div class="overflow-auto h-full" v-if="searchResult !== undefined">
      <NormalSeasonPanel
        v-if="'Normal' in searchResult && searchResult.Normal.ugc_season !== null"
        :normal-result="searchResult.Normal"
        :ugc-season="searchResult.Normal.ugc_season" />
      <NormalSinglePanel v-else-if="'Normal' in searchResult" :normal-result="searchResult.Normal" />
      <BangumiPanel v-else-if="'Bangumi' in searchResult" :bangumi-result="searchResult.Bangumi" />
    </div>
  </div>
</template>
