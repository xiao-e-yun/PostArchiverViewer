<script lang="ts" setup>
import { ChevronLeft } from "lucide-vue-next";
import { RouterLink } from "vue-router";

withDefaults(
  defineProps<{
    to?: string;
  }>(),
  {
    to: "/",
  },
);
</script>

<template>
  <RouterLink :to="to" class="pb-4 inline-flex group">
    <ChevronLeft class="group-hover:-translate-x-1 transition-transform" />
    <slot> Home </slot>
  </RouterLink>
</template>
