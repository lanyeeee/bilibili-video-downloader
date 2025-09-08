<script setup lang="ts">
import { AudioQuality, VideoQuality, CodecType } from '../../../bindings.ts'
import { useStore } from '../../../store.ts'
import { VueDraggable } from 'vue-draggable-plus'
import ColorfulTag from '../../../components/ColorfulTag.vue'

const store = useStore()

const videoQualityNameMap: Map<VideoQuality, string> = new Map([
  ['240P', '240P 极速'],
  ['360P', '360P 流畅'],
  ['480P', '480P 标清'],
  ['720P', '720P 准高清'],
  ['720P60', '720P 60帧'],
  ['1080P', '1080P 高清'],
  ['AiRepair', 'AI智能修复'],
  ['1080P+', '1080P 高码率'],
  ['1080P60', '1080P 60帧'],
  ['4K', '4K 超高清'],
  ['HDR', 'HDR 真彩色'],
  ['Dolby', '杜比视界'],
  ['8K', '8K 超高清'],
])

const audioQualityNameMap: Map<AudioQuality, string> = new Map([
  ['64K', '64K'],
  ['132K', '132K'],
  ['192K', '192K'],
  ['Dolby', '杜比全景声'],
  ['HiRes', 'Hi-Res 无损'],
])

const codecTypeNameMap: Map<CodecType, string> = new Map([
  ['AVC', 'AVC (H.264)'],
  ['HEVC', 'HEVC (H.265)'],
  ['AV1', 'AV1'],
])
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col gap-row-2">
    <div class="flex gap-2">
      <span class="w-15 font-bold">主要内容</span>
      <n-checkbox class="w-22" v-model:checked="store.config.download_video">下载视频</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="store.config.download_audio">下载音频</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="w-15 font-bold">视频处理</span>
      <n-tooltip placement="top" trigger="hover">
        <div>自动合并音频和视频</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="store.config.auto_merge">自动合并</n-checkbox>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>如果视频有章节分段，则将章节信息嵌入mp4文件的元数据中</div>
        <div>使视频在各类播放器中支持章节导航(例如进度条分段)</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="store.config.embed_chapter">标记章节</n-checkbox>
        </template>
      </n-tooltip>
      <n-tooltip placement="top" trigger="hover">
        <div>将视频的广告部分以章节的形式嵌入mp4文件的元数据中</div>
        <div>可以实现自动跳过广告(如果播放器支持的话)</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="store.config.embed_skip">标记广告</n-checkbox>
        </template>
      </n-tooltip>
    </div>

    <div class="flex gap-2">
      <span class="w-15 font-bold">下载弹幕</span>
      <n-checkbox class="w-22" v-model:checked="store.config.download_xml_danmaku">xml弹幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="store.config.download_ass_danmaku">ass弹幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="store.config.download_json_danmaku">json弹幕</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="w-15 font-bold">其他内容</span>
      <n-checkbox class="w-22" v-model:checked="store.config.download_subtitle">下载字幕</n-checkbox>
      <n-checkbox class="w-22" v-model:checked="store.config.download_cover">下载封面</n-checkbox>
    </div>

    <div class="flex gap-2">
      <span class="w-14 w-15 font-bold">元数据</span>
      <n-tooltip placement="top" trigger="hover">
        <div>还会顺便下载poster和fanart(如果有的话)</div>
        <template #trigger>
          <n-checkbox class="w-22" v-model:checked="store.config.download_nfo">nfo刮削</n-checkbox>
        </template>
      </n-tooltip>
      <n-checkbox class="w-22" v-model:checked="store.config.download_json">json刮削</n-checkbox>
    </div>

    <div class="flex flex-justify-between">
      <div>
        <span class="whitespace-nowrap font-bold">画质优先级</span>
        <div class="overflow-auto overflow-x-hidden h-36">
          <VueDraggable
            :force-fallback="true"
            :fallback-on-body="true"
            :animation="300"
            v-model="store.config.video_quality_priority"
            ghostClass="draggable-ghost"
            class="select-none flex flex-col gap-2">
            <ColorfulTag
              class="whitespace-nowrap cursor-move"
              color="blue"
              v-for="videoQuality in store.config.video_quality_priority"
              :key="videoQuality">
              {{ videoQualityNameMap.get(videoQuality) || videoQuality }}
            </ColorfulTag>
          </VueDraggable>
        </div>
      </div>

      <div>
        <span class="whitespace-nowrap font-bold">音质优先级</span>
        <VueDraggable
          :force-fallback="true"
          :fallback-on-body="true"
          v-model="store.config.audio_quality_priority"
          :animation="300"
          ghostClass="draggable-ghost"
          class="select-none flex flex-col gap-2">
          <ColorfulTag
            class="whitespace-nowrap cursor-move"
            color="blue"
            v-for="audioQuality in store.config.audio_quality_priority"
            :key="audioQuality">
            {{ audioQualityNameMap.get(audioQuality) || audioQuality }}
          </ColorfulTag>
        </VueDraggable>
      </div>

      <div>
        <span class="whitespace-nowrap font-bold">编码优先级</span>
        <VueDraggable
          :force-fallback="true"
          :fallback-on-body="true"
          v-model="store.config.codec_type_priority"
          :animation="300"
          ghostClass="draggable-ghost"
          class="select-none flex flex-col gap-2">
          <ColorfulTag
            class="whitespace-nowrap cursor-move"
            color="blue"
            v-for="codecType in store.config.codec_type_priority"
            :key="codecType">
            {{ codecTypeNameMap.get(codecType) || codecType }}
          </ColorfulTag>
        </VueDraggable>
      </div>
    </div>
  </div>
</template>

<style scoped>
.draggable-ghost {
  opacity: 0.5;
}
</style>
