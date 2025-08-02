<script setup lang="ts">
import AppContent from './AppContent.vue'
import { GlobalThemeOverrides } from 'naive-ui'

const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#0EA5E9',
    primaryColorHover: '#36BFF5',
    primaryColorPressed: '#027FC2',
    primaryColorSuppl: '#36BFF5',
    borderRadius: '4px',
    borderRadiusSmall: '3px',
    heightMedium: '32px',
  },
  Tabs: {
    tabGapSmallLine: '10px',
    tabPaddingSmallLine: '6px 8px',
  },
  Button: {
    paddingSmall: '0 8px',
    paddingMedium: '0 12px',
  },
  Radio: {
    buttonColorActive: '#0EA5E9',
    buttonTextColorActive: '#FFF',
  },
  Dropdown: {
    borderRadius: '5px',
    padding: '6px 2px',
    optionColorHover: '#0EA5E9',
    optionTextColorHover: '#FFF',
    optionHeightMedium: '28px',
  },
}
</script>

<template>
  <n-config-provider :theme-overrides="themeOverrides">
    <n-dialog-provider>
      <n-modal-provider>
        <n-notification-provider placement="bottom-right" :max="3">
          <n-message-provider>
            <AppContent />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>
