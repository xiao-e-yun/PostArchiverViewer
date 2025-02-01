<script lang="ts" setup>
import Image from "@/components/image/Image.vue";
import { Badge } from "@/components/ui/badge";
import Card from "@/components/ui/card/Card.vue";
import { Separator } from "@/components/ui/separator";
import type { PostAPI } from "@/api";
import { useFetch } from "@vueuse/core";
import { ChevronLeft } from "lucide-vue-next";
import { marked } from "marked";
import { computed } from "vue";
import { RouterLink, useRoute } from "vue-router";
import type { FileMetaJson } from "@api/FileMetaJson";
import {
  DialogTrigger,
} from "@/components/ui/dialog";
import DialogImage from "@/components/DialogImage.vue";

let lastId = "0" as string;
const route = useRoute();
const id = computed(() => route.params.post as string | undefined);
const url = computed(
  () => `/api/post?post=${parseInt((lastId = id.value ?? lastId))}`
);
const { data: post, isFetching } = useFetch(url, {
  refetch: true,
}).json<PostAPI>();

const contents = computed(() => {
  const $post = post.value;
  if (!$post) return [];

  let textList = [];
  let contents: (string | FileMetaJson)[] = [];
  for (const c of $post.content) {
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
  return contents;
});

const author = computed(() => post.value!.author);

const tags = computed(() => post.value!.tags);

function getStyleByFileExtra(extra: FileMetaJson["extra"]) {
  if (!hasExtra(extra)) return {};
  const width = parseInt(extra.width as string);
  const height = parseInt(extra.height as string);
  return {
    aspectRatio: width / height,
  };
}
function hasExtra(extra: FileMetaJson["extra"]) {
  return extra && (extra.width || extra.height);
}

// TODO COMMENT
</script>

<template>
  <template v-if="isFetching">
    <h1 class="text-4xl font-bold my-4">Loading...</h1>
  </template>
  <template v-else-if="post">
    <RouterLink :to="`/author/${author.id}`" class="flex p-2">
      <ChevronLeft /> <span class="font-bold">{{ author.name }}</span>
    </RouterLink>
    <div class="capitalize">
      <h1 class="md:text-4xl text-2xl mt-4 font-bold text-center">
        {{ post.title }}
      </h1>

      <div class="flex gap-2 my-4">
        <RouterLink v-if="author" :to="`/author/${author.id}`">
          <Badge title="Author">{{ author.name }}</Badge>
        </RouterLink>
        <a v-if="post.source" :href="post.source">
          <Badge variant="secondary">source</Badge>
        </a>
      </div>
      <div class="flex gap-2 my-4">
        <Badge
          class="bg-blue-300 dark:bg-blue-600"
          variant="secondary"
          title="Updated"
          >{{ new Date(post.updated).toLocaleString() }}</Badge
        >
        <Badge
          class="bg-rose-300 dark:bg-rose-500"
          variant="secondary"
          title="Published"
          >{{ new Date(post.published).toLocaleString() }}</Badge
        >
        <Badge v-for="tag in tags" variant="secondary">{{ tag.name }}</Badge>
      </div>
    </div>
    <Separator class="my-4" />
    <div
      class="flex flex-col gap-4 pt-4 lg:w-[1024px] mx-auto"
      :class="$style.content"
    >
      <template v-for="content in contents">
        <div v-if="typeof content === 'string'" v-html="content" />
        <Card
          v-else
          class="m-auto overflow-hidden max-h-[80vh] max-w-full relative"
          :style="getStyleByFileExtra(content.extra)"
        >
          <svg
            v-if="hasExtra(content.extra)"
            :width="content.extra.width"
            :height="content.extra.height"
          />

          <DialogImage
            v-if="content.mime.startsWith('image')"
            :aspect="getStyleByFileExtra(content.extra).aspectRatio"
            :src="content.url"
            class="p-0"
          >
            <DialogTrigger as="div">
              <Image
                :width="100"
                :src="content.url"
                :aspect="getStyleByFileExtra(content.extra).aspectRatio"
                class="object-cover max-h-[80vh] w-full absolute inset-0"
              />
            </DialogTrigger>
          </DialogImage>

          <video
            v-else-if="content.mime.startsWith('video')"
            :src="content.url"
            controls
          />

          <audio
            v-else-if="content.mime.startsWith('audio')"
            :src="content.url"
          />

          <a
            v-else
            :href="content.url"
            target="_blank"
            rel="noopener noreferrer"
          >
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
  color: rgb(147 197 253 / var(--tw-text-opacity, 1));
}

.content a:hover {
  text-decoration: underline;
}
</style>
