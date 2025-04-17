<script setup lang="tsx">
import { computed, inject } from "vue";
import { postKey } from "./utils";
import type { Comment } from "@api/Comment";
import Separator from "../ui/separator/Separator.vue";

const { post } = inject(postKey)!;

const comments = computed(() => {
  const $post = post.value;
  if (!$post) return [];

  const parseComments = (comments: Comment[], level = 0) => {
    const flattened: [Comment, number][] = [];

    for (const comment of comments) {
      flattened.push([comment, level]);
      const replies = parseComments(comment.replies ?? [], level + 1);
      flattened.push(...replies);
    }

    return flattened;
  };

  return parseComments($post.comments);
});
</script>

<template>
  <div v-if="comments.length" class="flex flex-col">
    <Separator label="Comment" class="text-lg mt-6" />
    <div
      v-for="([comment, level], index) in comments"
      :key="index"
      class="flex px-2"
    >
      <div v-for="index in level" :key="index" class="mx-2 border-l" />
      <div
        class="flex flex-col"
        :style="{ paddingTop: level == 0 ? '2rem' : '1rem' }"
      >
        <span class="rounded-sm bg-secondary w-fit px-1"
          >{{ comment.user }}:</span
        >
        <p class="pl-2">{{ comment.text }}</p>
      </div>
    </div>
  </div>
</template>
