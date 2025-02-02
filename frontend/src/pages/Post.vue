<script lang="ts" setup>
import Image from "@/components/image/Image.vue";
import { Badge } from "@/components/ui/badge";
import Card from "@/components/ui/card/Card.vue";
import { Separator } from "@/components/ui/separator";
import type { PostAPI } from "@/api";
import { useFetch } from "@vueuse/core";
import { ChevronLeft } from "lucide-vue-next";
import { marked } from "marked";
import { computed, nextTick } from "vue";
import { RouterLink, useRoute } from "vue-router";
import type { FileMetaJson } from "@api/FileMetaJson";
import { DialogTrigger } from "@/components/ui/dialog";
import DialogImage from "@/components/DialogImage.vue";
import Skeleton from "@/components/ui/skeleton/Skeleton.vue";
import { getUrlWithParams } from "@/utils";
import { useLazyLoad } from "@/lazyload";

let lastId = "0" as string;
const route = useRoute();
const id = computed(() => route.params.post as string | undefined);
const url = computed(
  () =>
    getUrlWithParams("/api/post", { post: (lastId = id.value ?? lastId) }).href
);
const {
  data: post,
  statusCode,
  isFetching,
} = useFetch(url, {
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
  
  nextTick(()=>useLazyLoad().update())
  return contents;
});

const author = computed(() => post.value?.author);

const tags = computed(() => post.value?.tags);

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
  <template v-if="statusCode === 404">
    <h1>Post not found</h1>
    <RouterLink to="/">Home</RouterLink>
  </template>
  <template v-else>
    <RouterLink :to="author ? `/author/${author.id}` : `/`" class="flex p-2">
      <ChevronLeft />
      <Skeleton v-if="!author" class="w-20" />
      <span class="font-bold" v-else>{{ author.name }}</span>
    </RouterLink>
    <div class="capitalize">
      <h1 class="md:text-4xl text-2xl mt-4 font-bold text-center">
        <Skeleton v-if="!post" class="w-[12em] h-[1.1em] mx-auto" />
        <template v-else>{{ post.title }}</template>
      </h1>

      <div class="flex gap-2 my-4">
        <RouterLink v-if="author" :to="`/author/${author.id}`">
          <Badge title="Author">{{ author.name }}</Badge>
        </RouterLink>
        <Skeleton v-else class="rounded-full w-24 my-px" />
        <a v-if="post?.source" :href="post.source">
          <Badge variant="secondary">source</Badge>
        </a>
      </div>
      <div class="flex gap-2 my-4">
        <Skeleton v-if="!post" v-for="_ in 2" class="rounded-full w-[120px]" />
        <template v-else>
          <Badge
            class="bg-blue-300 dark:bg-blue-600"
            variant="secondary"
            title="Updated"
            >{{ new Date(post.updated).toLocaleString("zh-CN") }}</Badge
          >
          <Badge
            class="bg-rose-300 dark:bg-rose-500"
            variant="secondary"
            title="Published"
            >{{ new Date(post.published).toLocaleString("zh-CN") }}</Badge
          >
        </template>
        <Skeleton
          v-if="tags === undefined"
          v-for="width in ['20', '16', '10']"
          class="rounded-full"
          :class="'w-' + width"
        />
        <Badge v-else v-for="tag in tags" variant="secondary">{{
          tag.name
        }}</Badge>
      </div>
    </div>
    <Separator class="my-4" />
    <div
      class="flex flex-col gap-4 pt-4  px-4 lg:w-[1024px] mx-auto md:border-x md:px-6"
      :class="$style.content"
    >
      <template v-if="isFetching">
        <Skeleton style="width: 89%; height: 1em" />
        <Skeleton style="width: 26%; height: 1em" />
        <Skeleton style="width: 61%; height: 1em" />
        <Skeleton style="width: 21%; height: 1em" />
        <Skeleton style="aspect-ratio: 0.8; height: 80vh; margin: auto" />
      </template>
      <template v-else v-for="content in contents">
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
            class="lazy"
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
