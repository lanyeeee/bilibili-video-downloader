<script setup lang="tsx">
import { LogEvent, LogLevel, events, commands } from '../bindings.ts'
import { useNotification } from 'naive-ui'
import { onMounted, ref, watch, computed } from 'vue'
import { appDataDir } from '@tauri-apps/api/path'
import { path } from '@tauri-apps/api'
import { useStore } from '../store.ts'
import { darkTheme } from 'naive-ui'

type LogRecord = LogEvent & { id: number; formatedLog: string }

const store = useStore()

const notification = useNotification()

const showing = defineModel<boolean>('showing', { required: true })

let nextLogRecordId = 1

const logRecords = ref<LogRecord[]>([])
const searchText = ref<string>('')
const selectedLevel = ref<LogLevel>('INFO')
const logsDirSize = ref<number>(0)

onMounted(async () => {
  const result = await commands.getLogsDirSize()
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  // 检查日志目录大小
  if (result.data > 50 * 1024 * 1024) {
    notification.warning({
      title: '日志目录大小超过50MB，请及时清理日志文件',
      description: () => (
        <>
          <div>
            点击左下角的 <span class="bg-gray-2 px-1">日志</span> 按钮
          </div>
          <div>
            里边有 <span class="bg-gray-2 px-1">打开日志目录</span> 按钮
          </div>
          <div>
            你也可以在里边取消勾选 <span class="bg-gray-2 px-1">输出文件日志</span>
          </div>
          <div>这样将不再产生文件日志</div>
        </>
      ),
    })
  }
})

const formatedLogsDirSize = computed<string>(() => {
  const units = ['B', 'KB', 'MB']
  let size = logsDirSize.value
  let unitIndex = 0

  while (size >= 1024 && unitIndex < 2) {
    size /= 1024
    unitIndex++
  }

  // 保留两位小数
  return `${size.toFixed(2)} ${units[unitIndex]}`
})
const filteredLogs = computed<LogRecord[]>(() => {
  return logRecords.value.filter(({ level, formatedLog }) => {
    // 定义日志等级的优先级顺序
    const logLevelPriority = {
      TRACE: 0,
      DEBUG: 1,
      INFO: 2,
      WARN: 3,
      ERROR: 4,
    }
    // 首先按日志等级筛选
    if (logLevelPriority[level] < logLevelPriority[selectedLevel.value]) {
      return false
    }
    // 然后按搜索文本筛选
    if (searchText.value === '') {
      return true
    }

    return formatedLog.toLowerCase().includes(searchText.value.toLowerCase())
  })
})

watch(showing, async () => {
  if (showing.value) {
    const result = await commands.getLogsDirSize()
    if (result.status === 'error') {
      console.error(result.error)
      return
    }
    logsDirSize.value = result.data
  }
})

onMounted(async () => {
  await events.logEvent.listen(async ({ payload: logEvent }) => {
    const logRecord: LogRecord = {
      ...logEvent,
      id: nextLogRecordId++,
      formatedLog: formatLogEvent(logEvent),
    }
    logRecords.value.push(logRecord)

    const { level, fields } = logEvent
    if (level === 'ERROR') {
      notification.error({
        title: fields['err_title'] as string,
        description: fields['message'] as string,
        duration: 0,
      })
    }
  })
})

function formatLogEvent(logEvent: LogEvent): string {
  const { timestamp, level, fields, target, filename, line_number } = logEvent
  const fields_str = Object.entries(fields)
    .sort(([key1], [key2]) => key1.localeCompare(key2))
    .map(([key, value]) => `${key}=${value}`)
    .join(' ')
  return `${timestamp} ${level} ${target}: ${filename}:${line_number} ${fields_str}`
}

function getLevelStyles(level: LogLevel) {
  switch (level) {
    case 'TRACE':
      return 'text-gray-400'
    case 'DEBUG':
      return 'text-green-400'
    case 'INFO':
      return 'text-blue-400'
    case 'WARN':
      return 'text-yellow-400'
    case 'ERROR':
      return 'text-red-400'
  }
}

const logLevelOptions = [
  { value: 'TRACE', label: 'TRACE' },
  { value: 'DEBUG', label: 'DEBUG' },
  { value: 'INFO', label: 'INFO' },
  { value: 'WARN', label: 'WARN' },
  { value: 'ERROR', label: 'ERROR' },
]

function clearLogRecords() {
  logRecords.value = []
  nextLogRecordId = 1
}

async function showLogsDirInFileManager() {
  const logsDir = await path.join(await appDataDir(), '日志')
  const result = await commands.showPathInFileManager(logsDir)
  if (result.status === 'error') {
    console.error(result.error)
  }
}
</script>

<template>
  <n-modal v-model:show="showing" v-if="store.config !== undefined">
    <n-dialog
      :showIcon="false"
      :title="`日志目录总大小：${formatedLogsDirSize}`"
      @close="showing = false"
      style="width: 95%">
      <div class="mb-2 flex flex-wrap gap-2">
        <n-input-group class="w-100">
          <n-input size="small" v-model:value="searchText" placeholder="搜素日志..." clearable />
          <n-select size="small" v-model:value="selectedLevel" :options="logLevelOptions" style="width: 120px" />
        </n-input-group>

        <div class="flex flex-wrap gap-2 ml-auto items-center">
          <n-button size="small" @click="showLogsDirInFileManager">打开日志目录</n-button>
          <n-checkbox v-model:checked="store.config.enable_file_logger">输出文件日志</n-checkbox>
        </div>
      </div>

      <n-config-provider :theme="darkTheme" :theme-overrides="{ Scrollbar: { width: '8px' } }">
        <n-virtual-list
          class="h-[calc(100vh-300px)] overflow-hidden bg-gray-900"
          :item-size="42"
          item-resizable
          :hoverable="false"
          :items="filteredLogs"
          :scrollbar-props="{ trigger: 'none' }">
          <template #default="{ item: { level, formatedLog } }: { item: LogRecord }">
            <div :class="['py-1 px-3 hover:bg-white/10 whitespace-pre-wrap mr-4', getLevelStyles(level)]">
              {{ formatedLog }}
            </div>
          </template>
        </n-virtual-list>
      </n-config-provider>
      <div class="pt-1 flex">
        <n-button ghost class="ml-auto" size="small" type="error" @click="clearLogRecords">清空日志浏览器</n-button>
      </div>
    </n-dialog>
  </n-modal>
</template>
