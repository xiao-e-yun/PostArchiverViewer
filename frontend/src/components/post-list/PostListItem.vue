<script setup lang="ts">
import type { PostMiniJson } from "@api/PostMiniJson";
import DynamicImage from "../image/DynamicImage.vue";
import { Card, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { ImageOffIcon } from "lucide-vue-next";

defineProps<{
  post: PostMiniJson;
}>();
</script>

<template>
  <Card
    class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full overflow-hidden"
    as-child
  >
    <RouterLink :to="`/post/${post.id}`">
      <DynamicImage
        v-if="post.thumb"
        :src="post.thumb.url"
        :aspect="1 / 1"
        :width="30"
        class="aspect-square w-full object-cover"
      />
      <div v-else class="aspect-square">
        <ImageOffIcon class="w-full h-full p-4" :stroke-width="0.5" />
      </div>
      <CardHeader class="p-3 sm:p-4 md:p-6">
        <CardTitle class="text-base md:text-lg">{{ post.title }}</CardTitle>
        <CardDescription class="">{{
          new Date(post.updated).toLocaleString("zh-CN")
        }}</CardDescription>
      </CardHeader>
    </RouterLink>
  </Card>
</template>
