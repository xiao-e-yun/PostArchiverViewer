<script lang="ts" setup>
import { Badge } from '@/components/ui/badge';
import Card from '@/components/ui/card/Card.vue';
import { Separator } from '@/components/ui/separator';
import type { AuthorsAPI, File, PostAPI } from '@/types';
import { ChevronLeft } from 'lucide-vue-next';
import { marked } from 'marked';
import { ofetch } from 'ofetch';
import { RouterLink, useRoute } from 'vue-router';

const authors = await ofetch<AuthorsAPI>("/api/authors");

const pagePostId = parseInt(useRoute().params.id as string);
const post = await ofetch<PostAPI>("/api/post", { query: { post: pagePostId } });
const author = authors.find(author => author.id === post.author);

let textList = [];
let contents: (string | File)[] = [];
for (const c of post.content) {
  if (typeof c === "string") {
    const markedContent = marked(c);
    textList.push(markedContent);
  } else {
    if (textList.length) contents.push(textList.join(""));
    contents.push(c);
    textList = [];
  }
}
if (textList.length) contents.push(textList.join(""));

function getStyleByFileExtra(extra: File["extra"]) {
  if (!hasExtra(extra)) return {};
  return {
    aspectRatio: `${extra.width}/${extra.height}`,
  };
}
function hasExtra(extra: File["extra"]) {
  return extra && (extra.width || extra.height);
}

// TODO COMMENT
</script>

<template>
  <template v-if="post">
    <RouterLink v-if="author" :to="`/author/${author.id}`" class="flex p-2">
      <ChevronLeft /> <span class="font-bold">{{ author.name }}</span>
    </RouterLink>
    <div>
      <h1 class="md:text-4xl text-2xl mt-4 font-bold text-center">{{ post.title }}</h1>

      <div class="flex gap-2 my-4">
        <RouterLink v-if="author" :to="`/author/${author.id}`">
          <Badge title="Author">{{ author.name }}</Badge>
        </RouterLink>
        <a v-if="post.source" :href="post.source">
          <Badge variant="secondary">Source</Badge>
        </a>
      </div>
      <div class="flex gap-2 my-4">
        <Badge class="bg-blue-300 dark:bg-blue-600" variant="secondary" title="Updated">{{ new
          Date(post.updated).toLocaleString() }}</Badge>
        <Badge class="bg-rose-300 dark:bg-rose-500" variant="secondary" title="Published">{{ new
          Date(post.published).toLocaleString() }}</Badge>
      </div>
    </div>
    <Separator class="my-4" />
    <div class="flex flex-col gap-4 pt-4 lg:w-[1024px] mx-auto" :class="$style.content">
      <template v-for="content in contents">
        <div v-if="typeof content === 'string'" v-html="content" />
        <Card v-else class="m-auto overflow-hidden max-h-[80vh] max-w-full relative" :style="getStyleByFileExtra(content.extra)">

          <svg v-if="hasExtra(content.extra)" :width="content.extra.width" :height="content.extra.height" />

          <img v-if="content.mime.startsWith('image')" :src="content.url" decoding="async" loading="lazy"
            class="object-cover max-h-[80vh] w-auto absolute inset-0" />

          <video v-else-if="content.mime.startsWith('video')" :src="content.url" controls />

          <audio v-else-if="content.mime.startsWith('audio')" :src="content.url" />

          <a v-else :href="content.url" target="_blank" rel="noopener noreferrer">
            <Badge>{{ content.mime }}</Badge>
          </a>

        </Card>
      </template>
    </div>
  </template>
  <template v-else>
    <h1>Post not found</h1>
    <RouterLink to="/">Home</RouterLink>
  </template>
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
  color: rgb(147 197 253 / var(--tw-text-opacity, 1))
}

.content a:hover {
  text-decoration: underline;
}
</style>