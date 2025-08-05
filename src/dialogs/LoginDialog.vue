<script setup lang="ts">
import { commands, QrcodeData, QrcodeStatus } from '../bindings.ts'
import { ref, watch } from 'vue'
import { useMessage } from 'naive-ui'
import { useStore } from '../store.ts'
import icon from '../../src-tauri/icons/128x128.png'
import FloatLabelInput from '../components/FloatLabelInput.vue'

const store = useStore()

const message = useMessage()

const showing = defineModel<boolean>('showing', { required: true })

const currentTabName = ref<'cookie' | 'qrcode'>('cookie')

// 只要showing有任何变动，currentTabName就改为cookie
watch(showing, () => {
  currentTabName.value = 'cookie'
})

watch(
  () => store.config?.sessdata,
  async (value, oldValue) => {
    if (store.config === undefined) {
      return
    }
    if (oldValue !== undefined && oldValue !== '' && value === '') {
      // 如果旧的 sessdata 不为空，新的 sessdata 为空，相当于退出登录
      store.userInfo = undefined
      message.success('已退出登录')
      return
    } else if (value === undefined || value === '') {
      // 如果 sessdata 为空，说明用户没有登录
      return
    }
    const result = await commands.getUserInfo(value)
    if (result.status === 'error') {
      console.error(result.error)
      store.userInfo = undefined
      return
    }
    store.userInfo = result.data
    message.success('获取用户信息成功')
    showing.value = false
  },
)

watch([showing, currentTabName], async () => {
  if (currentTabName.value !== 'qrcode' || !showing.value) {
    return
  }
  // 如果当前选项卡是二维码，并且dialog正在显示，则生成二维码
  await generateQrcode()
})

const qrcodeData = ref<QrcodeData>()
const qrcodeStatus = ref<QrcodeStatus>()

async function generateQrcode() {
  const result = await commands.generateQrcode()
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  qrcodeData.value = result.data
  // 每隔一秒获取一次二维码状态，直到showing为false
  const interval = setInterval(async () => {
    if (!showing.value) {
      clearInterval(interval)
      return
    }
    await getQrcodeStatus()
    handleQrcodeStatus()
  }, 1000)
}

async function getQrcodeStatus() {
  if (qrcodeData.value === undefined) {
    return
  }
  const result = await commands.getQrcodeStatus(qrcodeData.value?.qrcode_key)
  if (result.status === 'error') {
    console.error(result.error)
    return
  }
  qrcodeStatus.value = result.data
}

function handleQrcodeStatus() {
  if (qrcodeStatus.value === undefined || store.config === undefined) {
    return
  }

  if (qrcodeStatus.value.code === 0) {
    const sessdata = qrcodeStatus.value.url.split('SESSDATA=')[1].split('&')[0]
    store.config.sessdata = encodeURIComponent(sessdata)
    showing.value = false
    message.success('登录成功')
  }
}
</script>

<template>
  <n-modal v-if="store.config !== undefined" v-model:show="showing">
    <n-dialog :showIcon="false" @close="showing = false" title="登录方式">
      <div class="flex flex-col">
        <n-tabs class="h-full" v-model:value="currentTabName" type="line" size="small" animated>
          <n-tab-pane name="cookie" tab="Cookie">
            <FloatLabelInput v-model:value="store.config.sessdata" label="SESSDATA" clearable />
          </n-tab-pane>
          <n-tab-pane name="qrcode" tab="二维码" display-directive="show:lazy">
            <div class="flex flex-col">
              二维码状态：{{ qrcodeStatus?.message }}
              <n-qr-code
                v-if="qrcodeData !== undefined"
                class="mx-auto my-4"
                error-correction-level="H"
                :size="360"
                :value="qrcodeData.url"
                :icon-src="icon"
                icon-background-color="transparent"
                :icon-size="96" />
              <div v-else class="w-90 h-90 mx-auto my-4 p-3" />
            </div>
          </n-tab-pane>
        </n-tabs>
      </div>
    </n-dialog>
  </n-modal>
</template>
