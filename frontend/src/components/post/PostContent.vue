<script setup lang="ts">
import { computed, inject } from "vue";
import { postKey } from "./utils";
import type { FileMeta } from "@api/FileMeta";
import { Marked } from "marked";
import PostFile from "./PostFile.vue";
import { useLazyLoad } from "@/lazyload";
import { throttle } from "lodash";
import { Skeleton } from "../ui/skeleton";

const { post } = inject(postKey)!;

const marked = new Marked({
  renderer: {
    link({ href, text }) {
      const redirectHref = `/api/redirect?url=${encodeURIComponent(href)}`;
      return `<a href="${redirectHref}" target="_blank" rel="noopener noreferrer">${text}</a>`;
    },
  },
});

const contents = computed(() => {
  const $post = post.value;
  if (!$post) return [];

  let textList = [];
  const contents: (string | FileMeta)[] = [];
  for (const c of $post.content) {
    if (typeof c === "string") {
      const markedContent = marked.parse(c);
      textList.push(markedContent);
    } else {
      if (textList.length) contents.push(textList.join(""));
      contents.push(c);
      textList = [];
    }
  }
  if (textList.length) contents.push(textList.join(""));

  return contents;
});

const isHtml = (content: string | FileMeta) => typeof content === "string";

const lazyload = useLazyLoad();
const update = throttle(() => lazyload.update(), 50, {
  leading: false,
  trailing: true,
});
</script>

<template>
  <div
    class="flex flex-col gap-4 p-4 lg:w-[1024px] mx-auto md:border-x md:px-6"
    :class="$style.content"
  >
    <template v-if="!post">
      <Skeleton style="width: 89%; height: 1em" />
      <Skeleton style="width: 26%; height: 1em" />
      <Skeleton style="width: 61%; height: 1em" />
      <Skeleton style="width: 21%; height: 1em" />
      <Skeleton style="aspect-ratio: 0.8; height: 80vh; margin: auto" />
    </template>
    <template v-for="content in contents" v-else :key="content">
      <!-- eslint-disable vue/no-v-html -->
      <div v-if="isHtml(content)" v-html="content" />
      <!--eslint-enable-->
      <PostFile v-else :file="content" @vue:mounted="update" />
    </template>
  </div>
</template>

<style module>
/* For rendering markdown content */
.content h1 {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 1rem 0;
}

.content h2 {
  font-size: 1.25rem;
  font-weight: 500;
  margin: 1rem 0;
}

.content h3 {
  font-size: 1rem;
  font-weight: 500;
  margin: 1rem 0;
}

.content h4 {
  font-size: 0.875rem;
  font-weight: 500;
  margin: 1rem 0;
}

.content a {
  color: rgb(147 197 253 / var(--tw-text-opacity, 1));
}

.content a:hover {
  text-decoration: underline;
}
</style>
