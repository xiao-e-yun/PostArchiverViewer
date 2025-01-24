<script setup lang="ts">
// @ts-ignore
import VueResponsiveImage from './responsive-image.vue'
import { computed } from 'vue';



const props = withDefaults(defineProps<{
  src: string
  width?: number | string
  aspect?: number | string
  dpr?: number | string
  format?: 'webp' | 'jpeg' | 'png' | null
}>(), {
  format: 'webp',
  width: 50,
  aspect: 16 / 9
});

const src = computed(() => {
  const url = props.src.startsWith("/") ? new URL(props.src, location.origin) : new URL(props.src)
  const dpr = props.dpr ? parseInt(props.dpr.toString()) : window.devicePixelRatio
  if (dpr !== 1) url.searchParams.set('dpr', dpr.toString())
  if (props.format) url.searchParams.set('output', props.format)

  const hasQuery = url.search.length > 0

  return url.toString() + (hasQuery ? '&' : '?') + 'w=%width%&h=%height%'
})
</script>

<template>
  <VueResponsiveImage :image-url="src" :image-ratio="aspect" :lazy-loading="true" :width-on-screen="width"
    v-bind="$attrs" />
</template>