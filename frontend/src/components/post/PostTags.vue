<script setup lang="ts">
import { computed, inject } from "vue";
import { postKey } from "./utils";
import { Skeleton } from "../ui/skeleton";
import { Badge } from "../ui/badge";

const { post } = inject(postKey)!;
const author = computed(() => post.value?.author);
const tags = computed(() => post.value?.tags);
</script>

<template>
  <!-- first -->
  <div class="flex gap-2 my-4">
    <RouterLink v-if="author" :to="`/author/${author.id}`">
      <Badge title="Author">{{ author.name }}</Badge>
    </RouterLink>
    <Skeleton v-else class="rounded-full w-24 h-[24px]" />
    <a v-if="post?.source" :href="post.source">
      <Badge variant="secondary">source</Badge>
    </a>
  </div>
  <!-- secondary -->
  <div class="flex gap-2 my-4">
    <template v-if="!post">
      <Skeleton
        v-for="i in 2"
        :key="i"
        class="rounded-full w-[120px] h-[22px]"
      />
    </template>
    <template v-else>
      <Badge
        class="bg-blue-300 dark:bg-blue-600"
        variant="secondary"
        title="Updated"
      >
        {{ new Date(post.updated).toLocaleString("zh-CN") }}
      </Badge>
      <Badge
        class="bg-rose-300 dark:bg-rose-500"
        variant="secondary"
        title="Published"
      >
        {{ new Date(post.published).toLocaleString("zh-CN") }}
      </Badge>
    </template>
    <template v-if="tags === undefined">
      <Skeleton class="rounded-full w-20" />
      <Skeleton class="rounded-full w-16" />
      <Skeleton class="rounded-full w-10" />
    </template>
    <Badge v-for="tag in tags" v-else :key="tag.id" variant="secondary">
      {{ tag.name }}
    </Badge>
  </div>
</template>
