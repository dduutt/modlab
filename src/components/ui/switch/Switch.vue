<script setup lang="ts">
import type { SwitchRootEmits, SwitchRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SwitchRoot,
  SwitchThumb,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<SwitchRootProps & {
  class?: HTMLAttributes['class']
  size?: 'sm' | 'default'
}>(), {
  size: 'default',
})

const emits = defineEmits<SwitchRootEmits>()

const delegatedProps = reactiveOmit(props, 'class', 'size')

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <SwitchRoot
    v-slot="slotProps"
    data-slot="switch"
    :data-size="size"
    v-bind="forwarded"
    :class="cn(
      'data-checked:bg-blue-600 data-unchecked:bg-gray-200 focus-visible:ring-2 focus-visible:ring-blue-400 shrink-0 rounded-full border-2 border-transparent data-[size=default]:h-5 data-[size=default]:w-9 data-[size=sm]:h-3.5 data-[size=sm]:w-6 peer group/switch relative inline-flex items-center transition-colors outline-none cursor-pointer data-disabled:cursor-not-allowed data-disabled:opacity-50',
      props.class,
    )"
  >
    <SwitchThumb
      data-slot="switch-thumb"
      class="bg-white rounded-full group-data-[size=default]/switch:size-4 group-data-[size=sm]/switch:size-3 group-data-[size=default]/switch:data-checked:translate-x-4 group-data-[size=sm]/switch:data-checked:translate-x-2.5 group-data-[size=default]/switch:data-unchecked:translate-x-0 group-data-[size=sm]/switch:data-unchecked:translate-x-0 pointer-events-none block ring-0 shadow-xs transition-transform"
    >
      <slot name="thumb" v-bind="slotProps" />
    </SwitchThumb>
  </SwitchRoot>
</template>
