<script setup lang="ts">
import { computed, inject } from "vue";
import { postKey } from "./utils";
import type { FileMeta } from "@api/FileMeta";
import { Marked } from "marked";
import PostFile from "./PostFile.vue";
import { useLazyLoad } from "@/lazyload";
import { throttle } from "lodash";
import { Skeleton } from "../ui/skeleton";
import PostComments from "./PostComments.vue";

const { post, relations } = inject(postKey)!;

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
      contents.push(relations.fileMetas.get(c)!);
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
  <div class="p-4 lg:w-[1024px] mx-auto md:border-x md:px-6">
    <div class="flex flex-col gap-4" :class="$style.content">
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

    <PostComments />
  </div>
</template>

<style module>
/* For rendering markdown content */
.content p + p {
  margin-top: 1rem;
}

.content hr {
  margin: 1em 0;
}

.content h1 {
  font-size: 2rem;
  font-weight: 600;
  margin: 1.25rem 0;
}

.content h2 {
  font-size: 1.75rem;
  font-weight: 550;
  margin: 1.2rem 0;
}

.content h3 {
  font-size: 1.5rem;
  font-weight: 500;
  margin: 1.125rem 0;
}

.content h4 {
  font-size: 1.25rem;
  font-weight: 500;
  margin: 1.1rem 0;
}

.content h5 {
  font-size: 1rem;
  font-weight: 450;
  margin: 1rem 0;
}

.content h6 {
  font-size: 0.875rem;
  font-weight: 400;
  margin: 0.75rem 0;
}

.content a {
  color: rgb(147 197 253 / var(--tw-text-opacity, 1));
}

.content a:hover {
  text-decoration: underline;
}

.content ul {
  padding: 0.5rem 1.5rem;
}

.content ul {
  list-style-type: disc;
}

.content ul ul {
  list-style-type: circle;
}

.content ul ul ul {
  list-style-type: square;
}

.content ol {
  padding: 0.5rem 2.5rem;
  list-style-type: decimal;
}

.content blockquote {
  border-left: hsl(var(--muted)) 0.25rem solid;
  padding: 0.5rem 0.8rem;
  margin: 0.5rem 0;
}

.content code {
  background: hsl(var(--muted));
  padding: 0 0.3rem;
  border-radius: 0.3rem;
}

.content pre {
  background: hsl(var(--muted));
  border-radius: 0.3rem;
  padding: 0.5rem;
  margin: 0.5rem 0;
}

.content table {
  margin: 1.5rem auto;
  border-radius: var(--radius);
}

.content thead {
  border-bottom: hsl(var(--muted)) 0.15rem solid;
}

.content tbody tr:nth-child(even) {
  background: hsl(var(--secondary));
}

.content th,
.content td {
  padding: 0.2rem 1.2rem;
}

.content img:not(:global(.vue-responsive-image)) {
  max-width: 35%;
  display: inline-block;
  object-fit: cover;
}
</style>
