<script setup lang="tsx">
import { ref } from 'vue'
import { useStore } from '../../../store.ts'

const store = useStore()

const dirFmt = ref<string>(store.config?.dir_fmt ?? '')
const dirFmtForPart = ref<string>(store.config?.dir_fmt_for_part ?? '')
const timeFmt = ref<string>(store.config?.time_fmt ?? '')

function AvailableFmtFields() {
  return (
    <>
      <div>
        可以用斜杠
        <span class="rounded bg-gray-500 px-1 select-all text-white">/</span>
        来分隔目录层级
      </div>
      <div class="text-blue">注意：最后一个层级是下载内容的文件名</div>
      <div class="font-semibold mt-2">
        <span>可用字段：</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">task_id</span>
        <span class="ml-2">下载任务ID</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">episode_type</span>
        <span class="ml-2">视频类型(Normal / Bangumi / Cheese)</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">aid</span>
        <span class="ml-2">av号</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">bvid</span>
        <span class="ml-2">bv号</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">cid</span>
        <span class="ml-2">cid</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">ep_id</span>
        <span class="ml-2">番剧/课程的章节号</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">duration</span>
        <span class="ml-2">视频时长，单位为秒</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">pub_ts</span>
        <span class="ml-2">视频发布的时间</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">collection_title</span>
        <span class="ml-2">
          合集名称，
          <span class="text-blue">对于单独(没有合集)的视频，合集名称就是视频名称</span>
        </span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">episode_title</span>
        <span class="ml-2">视频名称</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">episode_order</span>
        <span class="ml-2">在合集里的序号(从1起)</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">part_title</span>
        <span class="ml-2">分P名称</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">part_order</span>
        <span class="ml-2">分P序号</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">up_name</span>
        <span class="ml-2">up昵称</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">up_uid</span>
        <span class="ml-2">up的uid</span>
      </div>
      <div>
        <span class="rounded bg-gray-500 px-1 select-all">create_ts</span>
        <span class="ml-2">下载任务创建的时间</span>
      </div>
    </>
  )
}
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col">
    <n-config-provider
      :theme-overrides="{ Scrollbar: { color: 'rgba(255, 255, 255, 0.25)', colorHover: 'rgba(255, 255, 255, 0.3)' } }">
      <div class="font-bold">目录格式</div>
      <n-tooltip placement="top" trigger="hover" class="w-68vw max-h-35vh" :scrollable="true">
        <div>目录的命名格式，只作用于没分P的视频</div>
        <available-fmt-fields />
        <div class="font-semibold mt-2">例如格式</div>
        <div class="bg-gray-200 rounded-md p-1 text-black w-fit">{collection_title}/{episode_title}</div>
        <div class="font-semibold">
          <span>下载</span>
          <span class="text-blue mx-0.5">BV1eg411A7qd</span>
          <span>会产生一个文件夹</span>
        </div>
        <div class="bg-gray-200 rounded-md px-1 w-fit text-black">galgame纯音乐</div>
        <div class="font-semibold">文件夹内下载内容的文件名为</div>
        <div class="bg-gray-200 rounded-md px-1 w-fit text-black">【galgame纯音乐】 夜の向日葵</div>
        <template #trigger>
          <n-input
            v-model:value="dirFmt"
            size="small"
            @blur="store.config.dir_fmt = dirFmt"
            @keydown.enter="store.config.dir_fmt = dirFmt" />
        </template>
      </n-tooltip>

      <div class="mt-2 font-bold">目录格式(分P)</div>
      <n-tooltip placement="top" trigger="hover" class="w-68vw max-h-45vh" :scrollable="true">
        <div>有分P的视频用这个格式，而不是上面那个</div>
        <available-fmt-fields />
        <div class="font-semibold mt-2">例如格式</div>
        <div class="bg-gray-200 rounded-md p-1 text-black w-fit">
          {collection_title}/{episode_title}/{episode_title}-P{part_order} {part_title}
        </div>
        <div class="font-semibold">
          <span>下载</span>
          <span class="text-blue mx-0.5">BV1NMaXecEsv</span>
          <span>的</span>
          <span class="text-blue mx-0.5">P30</span>
          <span>会创建</span>
        </div>
        <div class="flex flex-col gap-1 text-black">
          <div class="bg-gray-200 rounded-md px-1 w-fit">动画、游戏专辑</div>
          <div class="bg-gray-200 rounded-md px-1 w-fit">【高音质】樱之诗OST</div>
        </div>
        <div class="font-semibold">两层文件夹，里面的下载内容的文件名为</div>
        <div class="bg-gray-200 rounded-md px-1 w-fit text-black">【高音质】樱之诗OST-P30 30.夢の歩みを見上げて</div>
        <template #trigger>
          <n-input
            v-model:value="dirFmtForPart"
            size="small"
            @blur="store.config.dir_fmt_for_part = dirFmtForPart"
            @keydown.enter="store.config.dir_fmt_for_part = dirFmtForPart" />
        </template>
      </n-tooltip>

      <div class="mt-2 font-bold">时间格式</div>
      <n-tooltip placement="top" trigger="hover">
        <div>
          影响字段
          <span class="rounded bg-gray-500 px-1 select-all">pub_ts</span>
          和
          <span class="rounded bg-gray-500 px-1 select-all">created_ts</span>
          的格式
        </div>
        <div>
          详细用法请看
          <n-a href="https://docs.rs/chrono/latest/chrono/format/strftime/index.html" target="_blank">
            https://docs.rs/chrono/latest/chrono/format/strftime/index.html
          </n-a>
        </div>
        <template #trigger>
          <n-input
            v-model:value="timeFmt"
            size="small"
            @blur="store.config.time_fmt = timeFmt"
            @keydown.enter="store.config.time_fmt = timeFmt" />
        </template>
      </n-tooltip>
    </n-config-provider>
  </div>
</template>
