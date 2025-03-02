<script setup lang="ts">
import { ChevronLeft } from "lucide-vue-next";
import { Skeleton } from "../ui/skeleton";
import type { AuthorJson } from "@api/AuthorJson";

defineProps<{
  author: AuthorJson | undefined;
}>();
</script>

<template>
  <RouterLink :to="author ? `/author/${author.id}` : `/`" class="flex p-2">
    <ChevronLeft />
    <Skeleton v-if="!author" class="w-20" />
    <span v-else class="font-bold">{{ author.name }}</span>
  </RouterLink>
</template>
