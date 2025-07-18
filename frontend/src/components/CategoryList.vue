<script setup lang="ts">
import { type UrlParams } from "@/utils";
import DynamicList from "./dynamic-list/DynamicList.vue";
import { Skeleton } from "./ui/skeleton";
import { Card, CardContent, CardDescription, CardTitle } from "./ui/card";
import { RouterLink } from "vue-router";
import { useLazyLoad } from "@/lazyload";
import DynamicImage from "./image/DynamicImage.vue";
import { ImageOffIcon } from "lucide-vue-next";
import { Badge } from "./ui/badge";
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    category: "authors" | "collections" | "tags" | "platforms";
    query?: UrlParams;
    controls?: boolean;
    limit?: number;
    inline?: boolean;
  }>(),
  {
    query: () => ({}),
    limit: undefined,
    controls: true,
    inline: false,
  },
);

const lazyload = useLazyLoad();
const hasThumb = computed(() =>
  ["authors", "collections"].includes(props.category),
);
</script>

<template>
  <!-- @vue-generic {import('@/api').Category} -->
  <DynamicList
    v-slot="{ list, itemPrePage, relations }"
    :url="`/api/${category}`"
    v-bind="props"
  >
    <div
      v-if="hasThumb"
      class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4"
      :class="{ 'inline-list': props.inline }"
    >
      <template v-if="!list">
        <Skeleton
          v-for="i in props.limit ?? itemPrePage"
          :key="i"
          class="aspect-video box-content [.inline-list>&:nth-child(n+3)]:max-md:hidden [.inline-list>&:nth-child(n+4)]:max-xl:hidden [.inline-list>&:nth-child(n+5)]:max-2xl:hidden [.inline-list>&:nth-child(n+6)]:hidden"
        />
      </template>
      <Card
        v-for="item in list"
        :key="item.id"
        class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full overflow-hidden [.inline-list>&:nth-child(n+3)]:max-md:hidden [.inline-list>&:nth-child(n+4)]:max-xl:hidden [.inline-list>&:nth-child(n+5)]:max-2xl:hidden [.inline-list>&:nth-child(n+6)]:hidden"
        as-child
      >
        <RouterLink :to="`/${category}/${item.id}`">
          <DynamicImage
            v-if="'thumb' in item"
            :src="relations.fileMetaPath(item.thumb!)!"
            :aspect="16 / 9"
            :width="30"
            class="aspect-video w-full object-cover opacity-50"
            @vue:mounted="() => lazyload.update()"
          />
          <div v-else class="aspect-video opacity-50">
            <ImageOffIcon class="w-full h-full p-4" :stroke-width="0.5" />
          </div>
          <CardTitle class="text-xl md:text-2xl absolute top-0 left-0 p-4">
            {{ (category === "authors" ? "@" : "") + item.name }}
          </CardTitle>
          <CardDescription class="absolute bottom-4 left-4">
            <Badge
              v-if="'updated' in item && item.updated"
              class="max-sm:hidden"
            >
              {{ new Date(item.updated).toLocaleString("zh-CN") }}
            </Badge>
          </CardDescription>
        </RouterLink>
      </Card>
    </div>
    <div
      v-else
      class="grid grid-cols-3 md:grid-cols-4 xl:grid-cols-6 gap-4"
      :class="{ 'inline-list': props.inline }"
    >
      <template v-if="!list">
        <Skeleton
          v-for="i in props.limit ?? itemPrePage"
          :key="i"
          class="h-[32px] md:h-[44px] [.inline-list>&:nth-child(n+4)]:max-md:hidden [.inline-list>&:nth-child(n+5)]:max-xl:hidden [.inline-list>&:nth-child(n+7)]:hidden"
        />
      </template>
      <Card
        v-for="item in list"
        :key="item.id"
        class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full overflow-ellipsis [.inline-list>&:nth-child(n+4)]:max-md:hidden [.inline-list>&:nth-child(n+5)]:max-xl:hidden [.inline-list>&:nth-child(n+7)]:hidden"
        as-child
      >
        <RouterLink :to="`/${category}/${item.id}`">
          <CardContent class="px-3 py-2 max-sm:text-xs">
            <CardTitle class="p-0 text-xs md:text-lg capitalize">
              <span v-if="'platform' in item && item.platform">
                {{ relations.platforms.get(item.platform)?.name || "Unknown" }}:
              </span>
              {{ (category === "tags" ? "#" : "") + item.name }}
            </CardTitle>
          </CardContent>
        </RouterLink>
      </Card>
    </div>
  </DynamicList>
</template>
