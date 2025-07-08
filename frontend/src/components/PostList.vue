<script setup lang="ts">
import { type UrlParams } from "@/utils";
import DynamicList from "./dynamic-list/DynamicList.vue";
import { Skeleton } from "./ui/skeleton";
import { Card, CardDescription, CardTitle } from "./ui/card";
import { RouterLink } from "vue-router";
import { useLazyLoad } from "@/lazyload";
import DynamicImage from "./image/DynamicImage.vue";
import { ImageOffIcon } from "lucide-vue-next";
import { Badge } from "./ui/badge";

const props = withDefaults(
  defineProps<{
    url: string;
    query?: UrlParams;
    controls?: boolean;
    limit?: number;
    inline?: boolean;
  }>(),
  {
    query: () => ({}),
    limit: undefined,
    controls: true,
  },
);
</script>

<template>
  <!-- @vue-generic {import("@api/PostPreview").PostPreview} -->
  <DynamicList v-slot="{ list, itemPrePage, relations }" v-bind="props">
    <div
      class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4"
      :class="{ 'inline-list': props.inline }"
    >
      <template v-if="!list">
        <Skeleton
          v-for="i in props.limit ?? itemPrePage"
          :key="i"
          class="aspect-square box-content [.inline-list>&:nth-child(n+3)]:max-md:hidden [.inline-list>&:nth-child(n+4)]:max-xl:hidden [.inline-list>&:nth-child(n+5)]:max-2xl:hidden [.inline-list>&:nth-child(n+6)]:hidden"
        />
      </template>
      <Card
        v-for="post in list"
        :key="post.id"
        class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full overflow-hidden [.inline-list>&:nth-child(n+3)]:max-md:hidden [.inline-list>&:nth-child(n+4)]:max-xl:hidden [.inline-list>&:nth-child(n+5)]:max-2xl:hidden [.inline-list>&:nth-child(n+6)]:hidden"
        as-child
      >
        <RouterLink :to="`/posts/${post.id}`">
          <DynamicImage
            v-if="post.thumb"
            :src="relations.fileMetaPath(post.thumb)!"
            :aspect="1 / 1"
            :width="30"
            class="aspect-square w-full object-cover opacity-50"
            @vue:mounted="() => useLazyLoad().update()"
          />
          <div v-else class="aspect-square opacity-50">
            <ImageOffIcon class="w-full h-full p-4" :stroke-width="0.5" />
          </div>
          <CardTitle
            class="text-lg md:text-xl font-bold absolute top-0 left-0 p-4"
            >{{ post.title }}</CardTitle
          >
          <CardDescription>
            <Badge class="absolute bottom-4 left-4">
              {{ new Date(post.updated).toLocaleString("zh-CN") }}
            </Badge>
          </CardDescription>
        </RouterLink>
      </Card>
    </div>
  </DynamicList>
</template>
