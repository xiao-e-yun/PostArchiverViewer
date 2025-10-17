<script lang="ts" setup>
import CategoryList from "@/components/CategoryList.vue";
import { ChevronLeft } from "lucide-vue-next";
import { RouterLink } from "vue-router";

defineProps<{
  category: "authors" | "collections" | "tags" | "platforms";
}>();
</script>

<template>
  <RouterLink to="/" class="pb-4 inline-flex">
    <ChevronLeft />
    Home
  </RouterLink>
  <h1 class="text-4xl font-bold mb-4 capitalize">
    {{ category }}
  </h1>
  <CategoryList
    :category="category"
    :limit="['tags', 'platforms'].includes(category) ? 100 : undefined"
  />
</template>
