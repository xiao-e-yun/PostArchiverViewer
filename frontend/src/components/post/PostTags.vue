<script setup lang="ts">
import { computed, inject } from "vue";
import { postKey } from "./utils";
import { Skeleton } from "../ui/skeleton";
import { Badge } from "../ui/badge";
import DoubleBadge from "../DoubleBadge.vue";
import type { Tag } from "post-archiver";

const { post, relations } = inject(postKey)!;
const tags = computed(() => post.value?.tags);
const authors = computed(() => post.value?.authors);
const collections = computed(() => post.value?.collections);
const platform = computed(
  () => post.value?.platform && relations.platforms.get(post.value.platform),
);
const getPlatform = (tag: Tag) => {
  const platform = relations.platforms.get(tag.platform!)!;
  return { name: platform.name, link: `/platforms/${platform.id}` };
};

const source = computed(() => {
  const source = post.value?.source;
  if (!source) return null;
  try {
    const url = new URL(source);
    if (!url.protocol.startsWith("http")) {
      return null;
    }
    return url.href;
  } catch (_) {
    return null;
  }
});
</script>

<template>
  <!-- first -->
  <div class="flex gap-2 my-4 flex-wrap">
    <template v-if="authors">
      <RouterLink
        v-for="author in authors"
        :key="author.id"
        :to="`/authors/${author.id}`"
        class="flex items-center gap-1"
      >
        <Badge title="Author">@{{ author.name }}</Badge>
      </RouterLink>
    </template>
    <Skeleton v-else class="rounded-full w-24 h-[24px]" />
    <RouterLink
      v-if="platform"
      :to="`/platforms/${platform.id}`"
      class="flex items-center gap-1"
    >
      <Badge>{{ platform.name }}</Badge>
    </RouterLink>
    <Skeleton v-else class="rounded-full w-16 h-[24px]" />
    <a v-if="source" :href="source">
      <Badge variant="secondary">Source</Badge>
    </a>
  </div>
  <!-- secondary -->
  <div class="flex gap-2 my-4 flex-wrap">
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
    <template v-if="!post">
      <Skeleton class="rounded-full w-20" />
      <Skeleton class="rounded-full w-16" />
      <Skeleton class="rounded-full w-10" />
    </template>
    <!-- Collections -->
    <RouterLink
      v-for="collection in collections"
      :key="collection.id"
      :to="`/collections/${collection.id}`"
    >
      <Badge>
        {{ collection.name }}
      </Badge>
    </RouterLink>
    <!-- Tags -->
    <RouterLink v-for="tag in tags" :key="tag.id" :to="`/tags/${tag.id}`">
      <DoubleBadge
        v-if="tag.platform"
        :link="`/tags/${tag.id}`"
        :secondary-link="getPlatform(tag).link"
      >
        <template #secondary> {{ getPlatform(tag).name }} </template>
        #{{ tag.name }}
      </DoubleBadge>

      <Badge v-else variant="secondary"> #{{ tag.name }} </Badge>
    </RouterLink>
  </div>
</template>
