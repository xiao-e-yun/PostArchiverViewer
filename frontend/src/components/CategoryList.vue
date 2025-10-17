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
import { computed } from "vue";
import { categoryBuilders, CategoryType } from "@/category";
import DoubleBadge from "./DoubleBadge.vue";

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

const categoryPrefix = Object.fromEntries(
  categoryBuilders.map((builder) => [builder.TYPE, builder.PREFIX] as const),
) as Record<CategoryType, string>;
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
      :style="
        category == 'collections' && 'row-gap: 2.5rem; margin-bottom: 2.5rem;'
      "
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
        class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full [.inline-list>&:nth-child(n+3)]:max-md:hidden [.inline-list>&:nth-child(n+4)]:max-xl:hidden [.inline-list>&:nth-child(n+5)]:max-2xl:hidden [.inline-list>&:nth-child(n+6)]:hidden"
        as-child
      >
        <RouterLink :to="`/${category}/${item.id}`">
          <DynamicImage
            v-if="'thumb' in item && item.thumb"
            :src="relations.fileMetaPath(item.thumb!)!"
            :aspect="16 / 9"
            :width="30"
            class="aspect-video w-full object-cover rounded-md opacity-50"
            @vue:mounted="() => lazyload.update()"
          />
          <div v-else class="aspect-video rounded-md opacity-50">
            <ImageOffIcon class="w-full h-full p-4" :stroke-width="0.5" />
          </div>

          <template v-if="category === 'collections'">
            <div
              class="w-full h-full absolute bg-background top-0 -z-10 rounded-md"
            />
            <div
              class="w-full h-full absolute bg-secondary/30 top-3 -z-20 rounded-md border"
            />
            <div
              class="w-full h-full absolute bg-secondary/30 top-6 -z-30 rounded-md border"
            />
            <CardTitle
              class="text-xl md:text-2xl absolute top-0 left-0 w-full h-full flex items-center justify-center p-4 text-center"
            >
              {{ categoryPrefix[category] + item.name }}
            </CardTitle>
          </template>

          <CardTitle
            v-else
            class="text-xl md:text-2xl absolute top-0 left-0 p-4"
          >
            {{ categoryPrefix[category] + item.name }}
            <CardDescription class="mt-2">
              <Badge
                v-if="'updated' in item && item.updated"
                class="max-sm:hidden"
              >
                {{ new Date(item.updated).toLocaleString("zh-CN") }}
              </Badge>
            </CardDescription>
          </CardTitle>
        </RouterLink>
      </Card>
    </div>
    <div
      v-else
      class="flex flex-wrap gap-2"
      :class="{ 'inline-list': props.inline }"
    >
      <template v-if="!list">
        <Skeleton
          v-for="i in props.limit ?? itemPrePage"
          :key="i"
          class="h-[32px] md:h-[44px] [.inline-list>&:nth-child(n+4)]:max-md:hidden [.inline-list>&:nth-child(n+5)]:max-xl:hidden [.inline-list>&:nth-child(n+7)]:hidden"
        />
      </template>
      <RouterLink
        v-for="item in list"
        :key="item.id"
        :to="`/${category}/${item.id}`"
      >
        <DoubleBadge
          class="text-md transition-transform hover:scale-105 hover:z-10"
        >
          {{ categoryPrefix[category] + item.name }}
          <template v-if="item.platform" #secondary>
            {{ relations.platforms.get(item.platform)?.name || "Unknown" }}
          </template>
        </DoubleBadge>
      </RouterLink>
    </div>
  </DynamicList>
</template>
