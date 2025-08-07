import type { InjectionKey, Ref } from 'vue'
import SearchPane from './panes/SearchPane/SearchPane.vue'

export const navDownloadButtonRefKey = Symbol() as InjectionKey<Ref<HTMLDivElement | undefined>>

export const searchPaneRefKey = Symbol() as InjectionKey<Ref<InstanceType<typeof SearchPane> | undefined>>
