<script setup lang="ts">
import type { AuthorJson } from "@api/AuthorJson";
import { Skeleton } from "../ui/skeleton";
import { Button } from "../ui/button";
import { Badge } from "../ui/badge";

defineProps<{
  author: AuthorJson | null;
  pending: boolean;
}>();
</script>

<template>
  <div class="pt-8 pb-4 flex flex-col gap-2">
    <template v-if="pending">
      <Skeleton class="h-11 md:h-16 w-[360px]" />
      <Skeleton class="h-6 w-64" />
    </template>
    <template v-else-if="author">
      <h1 class="text-4xl md:text-6xl py-0.5">{{ author.name }}</h1>
      <div v-for="link in author.links" :key="link.url" class="capitalize">
        <a :href="link.url">
          <Badge>{{ link.name }}</Badge>
        </a>
      </div>
    </template>
    <template v-else>
      <h1 class="text-4xl font-bold my-4">Author not found</h1>
      <RouterLink to="/">
        <Button>Home</Button>
      </RouterLink>
    </template>
  </div>
</template>
