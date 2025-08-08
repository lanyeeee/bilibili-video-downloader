<script setup lang="ts">
import { PhFolderOpen } from '@phosphor-icons/vue'
import { commands } from '../../../bindings.ts'
import { open } from '@tauri-apps/plugin-dialog'
import { useStore } from '../../../store.ts'

const store = useStore()

async function showDownloadDirInFileManager() {
  if (store.config === undefined) {
    return
  }

  const result = await commands.showPathInFileManager(store.config.download_dir)
  if (result.status === 'error') {
    console.error(result.error)
  }
}

async function selectDownloadDir() {
  if (store.config === undefined) {
    return
  }

  const selectedDirPath = await open({ directory: true })
  if (selectedDirPath === null) {
    return
  }

  store.config.download_dir = selectedDirPath
}
</script>

<template>
  <n-input-group class="box-border" v-if="store.config !== undefined">
    <n-input-group-label size="small">下载目录</n-input-group-label>
    <n-input v-model:value="store.config.download_dir" size="small" readonly @click="selectDownloadDir" />
    <n-button class="w-10" size="small" @click="showDownloadDirInFileManager">
      <template #icon>
        <n-icon size="20">
          <PhFolderOpen />
        </n-icon>
      </template>
    </n-button>
  </n-input-group>
</template>
