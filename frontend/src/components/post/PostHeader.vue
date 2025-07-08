<script setup lang="ts">
import { inject } from "vue";
import { postKey } from "./utils";
import PostTags from "./PostTags.vue";
import { Skeleton } from "../ui/skeleton";
import DynamicImage from "../image/DynamicImage.vue";
import { useLazyLoad } from "@/lazyload";

const { post, relations } = inject(postKey)!;
</script>

<template>
  <div class="capitalize relative">
    <h1 class="md:text-4xl text-2xl mt-4 font-bold text-center">
      <Skeleton v-if="!post" class="w-[12em] h-[32px] md:h-[40px] mx-auto" />
      <template v-else>{{ post.title }}</template>
    </h1>
    <PostTags />

    <div
      class="absolute -bottom-4 md:w-[calc(100%+4rem)] w-[calc(100%+2rem)] max-w-none -left-4 md:-left-8 -z-10 overflow-hidden border-b h-[calc(100%+4rem)]"
    >
      <DynamicImage
        v-if="post?.thumb"
        :width="10"
        :src="relations.fileMetaPath(post.thumb)!"
        class="object-cover object-center inset-0 w-full h-full scale-110 blur-md opacity-0 [&.loaded]:opacity-50 duration-300"
        @vue:mounted="() => useLazyLoad().update()"
      />
    </div>
  </div>
</template>
