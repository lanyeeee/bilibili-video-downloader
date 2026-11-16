import type { DownloadProgress, DownloadTaskState } from '../../bindings.ts'
import type { DeepReadonly } from 'vue'

export type ProgressData = DownloadProgress & {
  state: DownloadTaskState
  percentage: number
  stateIndicator: string
  taskIndicator: string
}

export function createProgressData(progress: DownloadProgress, state: DownloadTaskState): ProgressData {
  return {
    ...progress,
    state,
    percentage: 0,
    stateIndicator: getStateIndicator(state),
    taskIndicator: '',
  }
}

export function getStateIndicator(state: DownloadTaskState): string {
  switch (state) {
    case 'Pending':
      return '排队中'
    case 'Downloading':
      return '下载中'
    case 'Paused':
      return '已暂停'
    case 'Completed':
      return '下载完成'
    case 'Failed':
      return '下载失败'
  }
}

export function createProgressUpdate(
  current: DeepReadonly<ProgressData>,
  progress: DownloadProgress,
): Partial<ProgressData> {
  const next = { ...current, ...progress }
  let percentage = next.percentage
  let taskIndicator = next.taskIndicator

  const videoTask = next.video_task
  const audioTask = next.audio_task
  const videoProcessTask = next.video_process_task
  const danmakuTask = next.danmaku_task
  const subtitleTask = next.subtitle_task
  const coverTask = next.cover_task
  const nfoTask = next.nfo_task
  const jsonTask = next.json_task

  const danmakuSelected = danmakuTask.xml_selected || danmakuTask.ass_selected || danmakuTask.json_selected

  if (videoTask.selected && !videoTask.completed && videoTask.content_length > 0) {
    const chunkCount = videoTask.chunks.length
    const completedChunks = videoTask.chunks.filter((chunk) => chunk.completed).length
    percentage = (completedChunks / chunkCount) * 100
    taskIndicator = `视频分片 ${completedChunks}/${chunkCount}`
  } else if (audioTask.selected && !audioTask.completed && audioTask.content_length > 0) {
    const chunkCount = audioTask.chunks.length
    const completedChunks = audioTask.chunks.filter((chunk) => chunk.completed).length
    percentage = (completedChunks / chunkCount) * 100
    taskIndicator = `音频分片 ${completedChunks}/${chunkCount}`
  } else if (!videoProcessTask.completed) {
    const embedSelected = videoProcessTask.embed_chapter_selected || videoProcessTask.embed_skip_selected
    if (videoProcessTask.merge_selected && embedSelected) {
      percentage = 100
      taskIndicator = '自动合并+嵌入章节元数据'
    } else if (videoProcessTask.merge_selected) {
      percentage = 100
      taskIndicator = '自动合并'
    } else if (embedSelected) {
      percentage = 100
      taskIndicator = '嵌入章节元数据'
    }
  } else if (danmakuSelected && !danmakuTask.completed) {
    percentage = 100
    taskIndicator = '弹幕'
  } else if (subtitleTask.selected && !subtitleTask.completed) {
    percentage = 100
    taskIndicator = '字幕'
  } else if (coverTask.selected && !coverTask.completed) {
    percentage = 100
    taskIndicator = '封面'
  } else if (nfoTask.selected && !nfoTask.completed) {
    percentage = 100
    taskIndicator = 'nfo刮削'
  } else if (jsonTask.selected && !jsonTask.completed) {
    percentage = 100
    taskIndicator = 'json刮削'
  }

  return { ...progress, percentage, taskIndicator }
}
