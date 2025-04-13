<script setup lang="ts">
// @ts-expect-error: don't have type definitions for components
import VueResponsiveImage from "./responsive-image.vue";
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    src: string;
    width?: number | string;
    aspect?: number | string;
    dpr?: number | string;
    format?: "webp" | "jpeg" | "png" | null;
  }>(),
  {
    format: "webp",
    width: 50,
    dpr: undefined,
  },
);

const src = computed(() => {
  const url = props.src.startsWith("/")
    ? new URL(props.src, location.origin)
    : new URL(props.src);
  const dpr = props.dpr
    ? parseInt(props.dpr.toString())
    : window.devicePixelRatio;

  if (dpr !== 1) url.searchParams.set("dpr", dpr.toString());
  if (props.format) url.searchParams.set("output", props.format);

  url.searchParams.set("w", "__WIDTH__");
  if (props.aspect) {
    url.searchParams.set("h", "__HEIGHT__");
  }

  return url.toString();
});
</script>

<template>
  <VueResponsiveImage
    :image-url="src"
    :image-ratio="aspect"
    :lazy-loading="true"
    :width-on-screen="width"
    v-bind="$attrs"
  />
</template>
