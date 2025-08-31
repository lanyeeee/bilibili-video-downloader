<script setup lang="ts">
import { SelectBaseOption } from 'naive-ui/es/select/src/interface'
import { PreferAudioQuality, PreferVideoQuality } from '../../../bindings.ts'
import { useStore } from '../../../store.ts'

const store = useStore()

const videoQualitySelectOptions: SelectBaseOption<PreferVideoQuality>[] = [
  { label: '最高', value: 'Best' },
  { label: '240P', value: '240P' },
  { label: '360P', value: '360P' },
  { label: '480P', value: '480P' },
  { label: '720P', value: '720P' },
  { label: '720P 60帧', value: '720P60' },
  { label: '1080P', value: '1080P' },
  { label: '智能修复', value: 'AiRepair' },
  { label: '1080P 高码率', value: '1080P+' },
  { label: '1080P 60帧', value: '1080P60' },
  { label: '4K', value: '4K' },
  { label: 'HDR', value: 'HDR' },
  { label: '杜比视界', value: 'Dolby' },
  { label: '8K', value: '8K' },
]
const audioQualitySelectOptions: SelectBaseOption<PreferAudioQuality>[] = [
  { label: '最高', value: 'Best' },
  { label: '64K', value: '64K' },
  { label: '132K', value: '132K' },
  { label: '192K', value: '192K' },
  { label: '杜比全景声', value: 'Dolby' },
  { label: 'Hi-Res', value: 'HiRes' },
]
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col gap-row-2">
    <div class="flex gap-2">
      <span class="font-bold">主要内容</span>
      <n-checkbox v-model:checked="store.config.download_video">下载视频</n-checkbox>
      <n-checkbox v-model:checked="store.config.download_audio">下载音频</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="font-bold">视频处理</span>
      <n-tooltip placement="top" trigger="hover">
        <div>自动合并音频和视频</div>
        <template #trigger>
          <n-checkbox v-model:checked="store.config.auto_merge">自动合并</n-checkbox>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>如果视频有章节分段，则将章节信息嵌入mp4文件的元数据中</div>
        <div>使视频在各类播放器中支持章节导航(例如进度条分段)</div>
        <template #trigger>
          <n-checkbox v-model:checked="store.config.embed_chapter">标记章节</n-checkbox>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>将视频的广告部分以章节的形式嵌入mp4文件的元数据中</div>
        <div>可以实现自动跳过广告(如果播放器支持的话)</div>
        <template #trigger>
          <n-checkbox v-model:checked="store.config.embed_skip">标记广告</n-checkbox>
        </template>
      </n-tooltip>
    </div>

    <div class="flex gap-2">
      <span class="font-bold">下载弹幕</span>
      <n-checkbox class="w-22" v-model:checked="store.config.download_xml_danmaku">xml弹幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="store.config.download_ass_danmaku">ass弹幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="store.config.download_json_danmaku">json弹幕</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="font-bold">其他内容</span>
      <n-checkbox v-model:checked="store.config.download_subtitle">下载字幕</n-checkbox>
      <n-checkbox v-model:checked="store.config.download_cover">下载封面</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="w-14 font-bold">元数据</span>
      <n-tooltip placement="top" trigger="hover">
        <div>还会顺便下载poster和fanart(如果有的话)</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="store.config.download_nfo">nfo刮削</n-checkbox>
        </template>
      </n-tooltip>
      <n-checkbox class="w-22" v-model:checked="store.config.download_json">json刮削</n-checkbox>
    </div>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      <div>如果视频有对应的画质，则使用对应的画质，否则选择最高画质</div>
      <template #trigger>
        <div class="flex items-center">
          <span class="mr-2 whitespace-nowrap font-bold">优先画质</span>
          <n-select
            size="small"
            v-model:value="store.config.prefer_video_quality"
            :options="videoQualitySelectOptions" />
        </div>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      <div>如果视频有对应的音质，则使用对应的音质，否则选择最高音质</div>
      <template #trigger>
        <div class="flex items-center">
          <span class="mr-2 whitespace-nowrap font-bold">优先音质</span>
          <n-select
            size="small"
            v-model:value="store.config.prefer_audio_quality"
            :options="audioQualitySelectOptions" />
        </div>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      <div>如果视频有对应的编码，则使用对应的编码，否则按照 AVC > HEVC > AV1 的顺序选择编码</div>
      <template #trigger>
        <div>
          <span class="mr-2 font-bold">优先编码</span>
          <n-radio-group v-model:value="store.config.prefer_codec_type" size="small">
            <n-radio-button value="AVC">AVC</n-radio-button>
            <n-radio-button value="HEVC">HEVC</n-radio-button>
            <n-radio-button value="AV1">AV1</n-radio-button>
          </n-radio-group>
        </div>
      </template>
    </n-tooltip>
  </div>
</template>
