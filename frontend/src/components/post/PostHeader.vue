<script setup lang="ts">
import { inject } from "vue";
import PostAuthorBack from "./PostAuthorBack.vue";
import { postKey } from "./utils";
import PostTags from "./PostTags.vue";
import { Skeleton } from "../ui/skeleton";

const { post } = inject(postKey)!;
</script>

<template>
  <PostAuthorBack :author="post?.author" />
  <div class="capitalize">
    <h1 class="md:text-4xl text-2xl mt-4 font-bold text-center">
      <Skeleton v-if="!post" class="w-[12em] h-[32px] md:h-[40px] mx-auto" />
      <template v-else>{{ post.title }}</template>
    </h1>
    <PostTags />
  </div>
</template>
