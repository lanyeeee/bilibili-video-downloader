<script setup lang="ts">
import { PhDownloadSimple } from '@phosphor-icons/vue'

const props = defineProps<{
  startX: number
  startY: number
  endX: number
  endY: number
  onAnimationEnd: () => void
}>()

const cssVars = {
  '--start-x': `${props.startX}px`,
  '--start-y': `${props.startY}px`,
  '--end-x': `${props.endX}px`,
  '--end-y': `${props.endY}px`,
}
</script>

<template>
  <div class="fly-icon animate" :style="cssVars" @animationend="onAnimationEnd">
    <PhDownloadSimple :size="20" weight="fill" />
  </div>
</template>

<style scoped>
.fly-icon {
  position: fixed;
  z-index: 9999;
  border-radius: 50%;
  background-color: #0ea5e9;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  pointer-events: none;

  left: var(--start-x);
  top: var(--start-y);

  transform: translate(-50%, -50%);
  opacity: 1;
}

.fly-icon.animate {
  animation: fly-animation 0.8s;
}

@keyframes fly-animation {
  0% {
    opacity: 1;
    transform: translate(-50%, -50%) scale(1);
  }

  50% {
    opacity: 1;
    transform: translate(calc(-50% - 30px), calc(-50% - 60px)) scale(0.8);
  }

  100% {
    opacity: 0;
    transform: translate(calc(var(--end-x) - var(--start-x) - 50%), calc(var(--end-y) - var(--start-y) - 50%))
      scale(0.5);
  }
}
</style>
