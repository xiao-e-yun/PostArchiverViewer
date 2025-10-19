<script setup lang="ts">
import { useTitle } from "@vueuse/core";
import { capitalize } from "lodash";
import { computed, useSlots } from "vue";

const slots = useSlots();
const title = computed(() => {
  const title = "Post Archiver Viewer";
  if (!slots.default) return title;
  const subTitle = slots.default()[0].children as string;
  return `${capitalize(subTitle.trim())} - ${title}`;
});

useTitle(title);
</script>

<!-- eslint-disable vue/valid-template-root -->
<template></template>
