<script setup lang="ts">
import { computed, ref } from 'vue'
import { InputInst, InputProps } from 'naive-ui'

const props = withDefaults(
  defineProps<{
    label: string
    size?: InputProps['size']
    type?: InputProps['type']
    clearable?: InputProps['clearable']
  }>(),
  {
    size: 'medium',
    type: 'text',
    clearable: false,
  },
)

const value = defineModel<InputProps['value']>('value', { required: true })

const focused = ref(false)
const NInputRef = ref<InputInst>()

const floating = computed(() => value.value !== '' || focused.value)

const translateY = computed(() => {
  if (props.size === 'tiny') {
    return 'translate-y-[-90%]'
  } else if (props.size === 'small') {
    return 'translate-y-[-120%]'
  } else if (props.size === 'medium') {
    return 'translate-y-[-140%]'
  } else if (props.size === 'large') {
    return 'translate-y-[-160%]'
  }
  return ''
})

defineExpose({ NInputRef })
</script>

<template>
  <n-input
    ref="NInputRef"
    :size="size"
    :type="type"
    :clearable="clearable"
    placeholder=""
    v-model:value="value"
    @focus="focused = true"
    @blur="focused = false">
    <template #prefix>
      <n-el
        tag="span"
        :class="[
          'float-label bg-white transition-all duration-200 ease-in-out',
          floating ? `text-0.75rem px-0.5 ${translateY}` : '',
        ]">
        {{ label }}
      </n-el>
    </template>
  </n-input>
</template>

<style scoped>
:deep(.n-input-wrapper) {
  @apply overflow-visible flex items-center;
}

:deep(.n-input__prefix) {
  @apply absolute leading-none z-2;
}

:deep(.n-input__input-el) {
  @apply relative z-3;
}

.n-input--focus .float-label,
.n-input:hover .float-label {
  color: var(--primary-color);
}
</style>
