<script setup lang="ts">
type Color =
  | 'red'
  | 'orange'
  | 'amber'
  | 'yellow'
  | 'lime'
  | 'green'
  | 'emerald'
  | 'teal'
  | 'cyan'
  | 'sky'
  | 'blue'
  | 'indigo'
  | 'violet'
  | 'purple'
  | 'fuchsia'
  | 'pink'
  | 'rose'
  | 'slate'
  | 'gray'
  | 'zinc'
  | 'neutral'
  | 'stone'

defineProps<{
  color: Color
}>()

const colorClasses = {
  red: 'bg-red-1 text-red-8',
  orange: 'bg-orange-1 text-orange-8',
  amber: 'bg-amber-1 text-amber-8',
  yellow: 'bg-yellow-1 text-yellow-8',
  lime: 'bg-lime-1 text-lime-8',
  green: 'bg-green-1 text-green-8',
  emerald: 'bg-emerald-1 text-emerald-8',
  teal: 'bg-teal-1 text-teal-8',
  cyan: 'bg-cyan-1 text-cyan-8',
  sky: 'bg-sky-1 text-sky-8',
  blue: 'bg-blue-1 text-blue-8 border-blue-6/50',
  indigo: 'bg-indigo-1 text-indigo-8',
  violet: 'bg-violet-1 text-violet-8',
  purple: 'bg-purple-1 text-purple-8',
  fuchsia: 'bg-fuchsia-1 text-fuchsia-8',
  pink: 'bg-pink-1 text-pink-8',
  rose: 'bg-rose-1 text-rose-8',
  slate: 'bg-slate-1 text-slate-8',
  gray: 'bg-gray-1 text-gray-8',
  zinc: 'bg-zinc-1 text-zinc-8',
  neutral: 'bg-neutral-1 text-neutral-8',
  stone: 'bg-stone-1 text-stone-8',
}
</script>

<template>
  <span :class="`px-2 rounded-sm ${colorClasses[color]}`">
    <slot />
  </span>
</template>
