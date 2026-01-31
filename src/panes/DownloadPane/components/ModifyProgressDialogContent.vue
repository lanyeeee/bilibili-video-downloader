<script setup lang="ts">
import { computed, DeepReadonly, onMounted, ref } from 'vue'
import {
  AvailableMediaFormats,
  CodecType,
  commands,
  GetAvailableMediaFormatsParams,
  VideoQuality,
  AudioQuality,
} from '../../../bindings'
import { ProgressData } from '../DownloadPane.vue'
import { SelectOption } from 'naive-ui'
import { getAudioQualityName, getCodecTypeName, getVideoQualityName } from '../../../utils'

type VideoFormatOption = SelectOption & {
  videoQuality: VideoQuality
  codecType: CodecType
}
type AudioFormatOption = SelectOption & { value: AudioQuality }

const props = defineProps<{
  p: DeepReadonly<ProgressData>
  destroyDialog: () => void
}>()

const progressData = ref<ProgressData>(JSON.parse(JSON.stringify(props.p)))
const availableMediaFormats = ref<AvailableMediaFormats>()

const videoFormatOptions = computed<VideoFormatOption[]>(() => {
  const priorityOption: VideoFormatOption = {
    label: '根据优先级自动选择',
    value: toVideoFormatSelectValue('Unknown', 'Unknown'),
    videoQuality: 'Unknown',
    codecType: 'Unknown',
  }

  if (availableMediaFormats.value === undefined) {
    return [priorityOption]
  }

  const videoQualitiesAndCodecTypes = availableMediaFormats.value.video_qualities_and_codec_types

  const options: VideoFormatOption[] = videoQualitiesAndCodecTypes.map((videoQualityAndCodecType) => {
    const videoQuality = videoQualityAndCodecType.video_quality
    const codecType = videoQualityAndCodecType.codec_type
    return {
      label: `${getVideoQualityName(videoQuality)} - ${getCodecTypeName(codecType)}`,
      value: toVideoFormatSelectValue(videoQuality, codecType),
      videoQuality: videoQuality,
      codecType: codecType,
    }
  })

  return [priorityOption, ...options]
})

const audioFormatOptions = computed<AudioFormatOption[]>(() => {
  const priorityOption: AudioFormatOption = {
    label: '根据优先级自动选择',
    value: 'Unknown',
  }

  if (availableMediaFormats.value === undefined) {
    return [priorityOption]
  }

  const options: AudioFormatOption[] = availableMediaFormats.value.audio_qualities.map((quality) => {
    return { label: getAudioQualityName(quality), value: quality }
  })

  return [priorityOption, ...options]
})

onMounted(async () => {
  const { episode_type, bvid, cid, ep_id } = progressData.value

  let params: GetAvailableMediaFormatsParams | undefined
  if (episode_type === 'Normal') {
    if (bvid === null) {
      return
    }
    params = { Normal: { bvid, cid } }
  } else if (episode_type === 'Bangumi') {
    params = { Bangumi: { cid } }
  } else if (episode_type === 'Cheese') {
    if (ep_id === null) {
      return
    }
    params = { Cheese: { ep_id } }
  }

  if (params === undefined) {
    return
  }

  const result = await commands.getAvailableMediaFormats(params)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }

  availableMediaFormats.value = result.data
})

async function restartDownloadTask() {
  await commands.restartDownloadTask({
    task_id: progressData.value.task_id,

    video_task_selected: progressData.value.video_task.selected,
    audio_task_selected: progressData.value.audio_task.selected,
    merge_selected: progressData.value.video_process_task.merge_selected,
    embed_chapter_selected: progressData.value.video_process_task.embed_chapter_selected,
    embed_skip_selected: progressData.value.video_process_task.embed_skip_selected,
    subtitle_task_selected: progressData.value.subtitle_task.selected,
    xml_danmaku_selected: progressData.value.danmaku_task.xml_selected,
    ass_danmaku_selected: progressData.value.danmaku_task.ass_selected,
    json_danmaku_selected: progressData.value.danmaku_task.json_selected,
    cover_task_selected: progressData.value.cover_task.selected,
    nfo_task_selected: progressData.value.nfo_task.selected,
    json_task_selected: progressData.value.json_task.selected,

    video_quality: progressData.value.video_task.video_quality,
    codec_type: progressData.value.video_task.codec_type,
    audio_quality: progressData.value.audio_task.audio_quality,
  })

  props.destroyDialog()
}

function handleVideoFormatSelectUpdate(_value: string, option: VideoFormatOption) {
  progressData.value.video_task.video_quality = option.videoQuality
  progressData.value.video_task.codec_type = option.codecType
}

function toVideoFormatSelectValue(videoQuality: VideoQuality, codecType: CodecType) {
  return `${videoQuality} ${codecType}`
}
</script>

<template>
  <div class="flex flex-col gap-row-2">
    <div class="flex gap-2">
      <span class="w-15 font-bold">主要内容</span>
      <n-checkbox class="w-22" v-model:checked="progressData.video_task.selected">下载视频</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="progressData.audio_task.selected">下载音频</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="w-15 font-bold">视频处理</span>
      <n-tooltip placement="top" trigger="hover">
        <div>自动合并音频和视频</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="progressData.video_process_task.merge_selected">
            自动合并
          </n-checkbox>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>如果视频有章节分段，则将章节信息嵌入mp4文件的元数据中</div>
        <div>使视频在各类播放器中支持章节导航(例如进度条分段)</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="progressData.video_process_task.embed_chapter_selected">
            标记章节
          </n-checkbox>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>将视频的广告部分以章节的形式嵌入mp4文件的元数据中</div>
        <div>可以实现自动跳过广告(如果播放器支持的话)</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="progressData.video_process_task.embed_skip_selected">
            标记广告
          </n-checkbox>
        </template>
      </n-tooltip>
    </div>

    <div class="flex gap-2">
      <span class="w-15 font-bold">下载弹幕</span>
      <n-checkbox class="w-22" v-model:checked="progressData.danmaku_task.xml_selected">xml弹幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="progressData.danmaku_task.ass_selected">ass弹幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="progressData.danmaku_task.json_selected">json弹幕</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="w-15 font-bold">其他内容</span>
      <n-checkbox class="w-22" v-model:checked="progressData.subtitle_task.selected">下载字幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="progressData.cover_task.selected">下载封面</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="w-15 font-bold">元数据</span>
      <n-tooltip placement="top" trigger="hover">
        <div>还会顺便下载poster和fanart(如果有的话)</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="progressData.nfo_task.selected">nfo刮削</n-checkbox>
        </template>
      </n-tooltip>
      <n-checkbox class="w-22" v-model:checked="progressData.json_task.selected">json刮削</n-checkbox>
    </div>

    <div class="flex gap-2 justify-between">
      <div class="flex flex-col w-full">
        <span class="font-bold">画质</span>
        <n-select
          :consistent-menu-width="false"
          size="small"
          menu-size="small"
          :theme-overrides="{
            peers: {
              InternalSelectMenu: {
                height: '175px',
              },
            },
          }"
          :default-value="
            toVideoFormatSelectValue(progressData.video_task.video_quality, progressData.video_task.codec_type)
          "
          :options="videoFormatOptions"
          @update:value="handleVideoFormatSelectUpdate" />
      </div>

      <div class="flex flex-col w-full">
        <span class="font-bold">音质</span>
        <n-select
          :consistent-menu-width="false"
          size="small"
          menu-size="small"
          v-model:value="progressData.audio_task.audio_quality"
          :options="audioFormatOptions" />
      </div>
    </div>

    <n-button class="mt-2" type="primary" @click="restartDownloadTask">修改完毕，按照这个配置重新开始下载吧</n-button>
  </div>
</template>
