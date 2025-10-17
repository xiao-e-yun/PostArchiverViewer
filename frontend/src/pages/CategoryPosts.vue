<script lang="ts" setup>
import type { Category } from "@/api";
import AuthorAliases from "@/components/AuthorAliases.vue";
import DynamicImage from "@/components/image/DynamicImage.vue";
import Search from "@/components/search/Search.vue";
import { Badge } from "@/components/ui/badge";
import { useLazyLoad } from "@/lazyload";
import { useFetchWithCache, useRelations } from "@/utils";
import type { WithRelations } from "@api/WithRelations";
import { useRouteParams } from "@vueuse/router";
import { ChevronLeft } from "lucide-vue-next";
import { computed } from "vue";

export interface CategoryPostsContext {
  category: string;
}

const props = defineProps<CategoryPostsContext>();
const id = useRouteParams("id", "0", { transform: Number });

const url = computed(() => `/api/${props.category}/${id.value}`);
const { data } = useFetchWithCache<WithRelations<Category>>(
  "category-meta",
  url,
);

const relations = useRelations(data);
</script>

<template>
  <div class="flex flex-col gap-2 mb-8 relative capitalize">
    <RouterLink :to="`/${props.category}`" class="pb-4 inline-flex">
      <ChevronLeft />
      all {{ category }}
    </RouterLink>

    <template v-if="data">
      <RouterLink
        class="mr-auto ml-4 capitalize"
        :to="`/${props.category}/${id}`"
      >
        <h1 class="text-4xl underline underline-offset-8">{{ data.name }}</h1>
        <h1 class="text-lg mt-1">{{ props.category }}</h1>
      </RouterLink>
      <div class="flex justify-end gap-2 w-full">
        <AuthorAliases v-if="props.category === 'authors'" :id="id" />
        <Badge v-if="'source' in data!">
          <a v-if="data.source" :href="data.source"> Source </a>
        </Badge>
      </div>

      <div
        v-if="'thumb' in data && data.thumb"
        class="absolute -bottom-4 md:w-[calc(100%+4rem)] w-[calc(100%+2rem)] max-w-none -left-4 md:-left-8 -z-10 overflow-hidden border-b h-[calc(100%+3rem)]"
      >
        <DynamicImage
          :width="10"
          :src="relations.fileMetaPath(data.thumb)!"
          class="object-cover object-center inset-0 w-full h-full scale-110 blur-md opacity-0 [&.loaded]:opacity-50 duration-300"
          @vue:mounted="() => useLazyLoad().update()"
        />
      </div>
    </template>
  </div>
  <Search :defaults="{ [category]: [id] }" />
</template>
