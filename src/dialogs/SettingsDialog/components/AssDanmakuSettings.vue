<script setup lang="ts">
import { useStore } from '../../../store.ts'

const store = useStore()
</script>

<template>
  <div v-if="store.config !== undefined" class="flex flex-col gap-row-2">
    <n-tooltip placement="left" trigger="hover">
      弹幕使用的字体名称
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">字体</n-input-group-label>
          <n-input class="w-full" v-model:value="store.config.danmaku_config.font" size="small" />
        </n-input-group>
      </template>
    </n-tooltip>

    <div class="flex items-center gap-2 whitespace-nowrap">
      <n-tooltip placement="left" trigger="hover">
        弹幕字体大小
        <template #trigger>
          <n-input-group class="box-border">
            <n-input-group-label size="small">字体大小</n-input-group-label>
            <n-input-number
              class="w-full"
              v-model:value="store.config.danmaku_config.font_size"
              size="small"
              :min="0"
              :parse="(x: string) => Number(x)"
              :show-button="false" />
          </n-input-group>
        </template>
      </n-tooltip>

      <n-checkbox v-model:checked="store.config.danmaku_config.bold">加粗</n-checkbox>
    </div>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      弹幕的描边宽度，单位为像素
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">描边宽度</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.outline"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      弹幕的不透明度，越小越透明，越大越不透明
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">不透明度</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.alpha"
            size="small"
            :min="0"
            :max="1"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      弹幕在屏幕上的【持续时间】，单位为秒，可以有小数
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">在屏幕上的持续时间</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.duration"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      【正常弹幕的屏幕填充占比】，默认为 0.5，即半个屏幕
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">显示区域</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.float_percentage"
            size="small"
            :min="0"
            :max="1"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      <div>是一个比例数，用来计算平衡不同字体的宽度</div>
      <div>有的字体比较粗、比较宽，可以适当调大（如 1.4、1.6）</div>
      <div>有的字体比较细、比较窄，可以适当调小（如 1.0、1.2）</div>
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">宽度乘子</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.width_ratio"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      用来调整弹幕时间的水平距离，单位是像素，如果想拉开弹幕之间的距离，可以调大
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">最小水平间距</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.horizontal_gap"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      计算弹幕高度的数值，即【行高度/行间距】。数值越大，弹幕的垂直距离越大
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">行高</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.lane_size"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      弹幕时间轴偏移，大于0 会让弹幕延后，小于0 会让弹幕提前，单位为秒
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">延后</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.time_offset"
            size="small"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      渲染的屏幕分辨率，这个并不会影响渲染区域的大小，只影响字体的相对大小，
      <span class="text-blue">没有特殊原因不用改</span>
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">屏幕宽度</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.width"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>

    <n-tooltip placement="left" trigger="hover" class="w-20vw">
      渲染的屏幕分辨率，这个并不会影响渲染区域的大小，只影响字体的相对大小，
      <span class="text-blue">没有特殊原因不用改</span>
      <template #trigger>
        <n-input-group class="box-border">
          <n-input-group-label size="small">屏幕高度</n-input-group-label>
          <n-input-number
            class="w-full"
            v-model:value="store.config.danmaku_config.height"
            size="small"
            :min="0"
            :parse="(x: string) => Number(x)"
            :show-button="false" />
        </n-input-group>
      </template>
    </n-tooltip>
  </div>
</template>
