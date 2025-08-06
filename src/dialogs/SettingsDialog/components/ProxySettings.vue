<script setup lang="ts">
import { computed, ref } from 'vue'
import { useStore } from '../../../store.ts'

const store = useStore()

const proxyHost = ref<string>(store.config?.proxy_host ?? '')

const disableProxyHostAndPort = computed(() => store.config?.proxy_mode !== 'Custom')
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col gap-row-2">
    <n-radio-group v-model:value="store.config.proxy_mode" size="small">
      <n-radio-button value="NoProxy">直连</n-radio-button>
      <n-radio-button value="System">系统代理</n-radio-button>
      <n-radio-button value="Custom">自定义</n-radio-button>
    </n-radio-group>

    <n-input-group>
      <n-input-group-label size="small">http://</n-input-group-label>
      <n-input
        :disabled="disableProxyHostAndPort"
        v-model:value="proxyHost"
        size="small"
        placeholder=""
        @blur="store.config.proxy_host = proxyHost"
        @keydown.enter="store.config.proxy_host = proxyHost" />
      <n-input-group-label size="small">:</n-input-group-label>
      <n-input-number
        :disabled="disableProxyHostAndPort"
        v-model:value="store.config.proxy_port"
        size="small"
        placeholder=""
        :parse="(x: string) => parseInt(x)" />
    </n-input-group>
  </div>
</template>
