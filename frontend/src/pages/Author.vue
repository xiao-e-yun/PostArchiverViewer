<script lang="ts" setup>
import Image from "@/components/image/Image.vue";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import type { AuthorAPI } from "@/api";
import {
  breakpointsTailwind,
  useBreakpoints,
  useFetch,
  useLocalStorage,
  useUrlSearchParams,
} from "@vueuse/core";
import { ChevronLeft, ImageOff, LayoutList } from "lucide-vue-next";
import { useRoute, useRouter } from "vue-router";
import { computed, useTemplateRef, watch } from "vue";
import type { AuthorPostsJson } from "@api/AuthorPostsJson";
import { Skeleton } from "@/components/ui/skeleton";
import { getUrlWithParams } from "@/utils";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Pagination,
  PaginationEllipsis,
  PaginationFirst,
  PaginationLast,
  PaginationList,
  PaginationListItem,
  PaginationNext,
  PaginationPrev,
} from "@/components/ui/pagination";
import { Button } from "@/components/ui/button";
import { useLazyLoad } from "@/lazyload";

let lastId = "0" as string;

const route = useRoute();
const router = useRouter();
const id = computed(() => route.params.author as string | undefined);
const authorUrl = computed(
  () => `/api/author?author=${parseInt((lastId = id.value ?? lastId))}`
);
const { data: author, isFetching: isAuthorFetching } = useFetch(authorUrl, {
  refetch: true,
}).json<AuthorAPI>();

const postsPrePage = useLocalStorage(
  "post-archiver-viewer.posts-per-page",
  "20"
);
watch(postsPrePage, () => (page.value = 1));

const searchParams = useUrlSearchParams();
const page = computed({
  get: () => parseInt((searchParams.page as string | undefined) ?? "1"),
  set: (value: number | string) => {
    searchParams.page = value.toString();
    router.push({ query: searchParams });
  },
});

const postsUrl = computed(
  () =>
    getUrlWithParams("/api/posts", {
      author: parseInt((lastId = id.value ?? lastId)),
      limit: postsPrePage.value,
      page: page.value - 1,
    }).href
);

const { data: postsData, isFetching: isPostsFetching } = useFetch(postsUrl, {
  refetch: true,
}).json<AuthorPostsJson>();

const postsEl = useTemplateRef<HTMLDivElement>("postsList");
watch(postsEl, (el) => el && useLazyLoad().update());

const breakpoints = useBreakpoints(breakpointsTailwind);
const isSmall = breakpoints.smallerOrEqual("sm");
const siblingCount = computed(
  () =>
    ({
      sm: 1,
      md: 2,
      lg: 2,
      xl: 3,
      "2xl": 3,
      "": 0,
    }[breakpoints.active().value])
);
</script>

<template>
  <RouterLink to="/" class="flex p-2"> <ChevronLeft /> Home </RouterLink>
  <div class="pt-8 pb-4 flex flex-col gap-2">
    <template v-if="isAuthorFetching">
      <Skeleton class="h-11 md:h-16 w-[360px]" />
      <Skeleton class="h-6 w-64" />
    </template>
    <template v-else-if="author">
      <h1 class="text-4xl md:text-6xl py-0.5">{{ author.name }}</h1>
      <div v-for="link in author.links" class="capitalize">
        <a :href="link.url">
          <Badge>{{ link.name }}</Badge>
        </a>
      </div>
    </template>
    <template v-else>
      <h1 class="text-4xl font-bold my-4">Author not found</h1>
      <RouterLink to="/">
        <Badge>Home</Badge>
      </RouterLink>
    </template>
  </div>
  <div class="flex justify-end gap-2 flex-wrap">
    <Select v-model="postsPrePage">
      <SelectTrigger class="w-32">
        <LayoutList />
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="20">20</SelectItem>
        <SelectItem value="50">50</SelectItem>
        <SelectItem value="100">100</SelectItem>
      </SelectContent>
    </Select>

    <Pagination
      v-slot="{ page }"
      :total="postsData?.total ?? 0"
      :sibling-count="siblingCount"
      :show-edges="!isSmall"
      :items-per-page="parseInt(postsPrePage)"
      v-model:page="page"
    >
      <PaginationList v-slot="{ items }" class="flex items-center gap-1">
        <PaginationFirst />
        <PaginationPrev />

        <template v-for="(item, index) in items">
          <PaginationListItem
            v-if="item.type === 'page'"
            :key="index"
            :value="item.value"
            as-child
          >
            <Button
              class="w-10 h-10 p-0"
              :variant="item.value === page ? 'default' : 'outline'"
            >
              {{ item.value }}
            </Button>
          </PaginationListItem>
          <PaginationEllipsis v-else :key="item.type" :index="index" />
        </template>

        <PaginationNext />
        <PaginationLast />
      </PaginationList>
    </Pagination>
  </div>
  <Separator class="my-4" />
  <div
    v-if="isPostsFetching"
    class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4"
  >
    <Skeleton
      v-for="_ in parseInt(postsPrePage)"
      class="pt-[100%] h-[122px] box-content"
    />
  </div>
  <div
    v-else-if="postsData"
    ref="postsList"
    class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4"
  >
    <RouterLink v-for="post in postsData.posts" :to="`/post/${post.id}`">
      <Card
        class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full overflow-hidden"
      >
        <Image
          v-if="post.thumb"
          :src="post.thumb.url"
          :aspect="1 / 1"
          :width="30"
          class="aspect-square w-full object-cover"
        />
        <div v-else class="aspect-square">
          <ImageOff class="w-full h-full p-4" :stroke-width="0.5" />
        </div>
        <CardHeader class="p-3 sm:p-4 md:p-6">
          <CardTitle class="text-base md:text-lg">{{ post.title }}</CardTitle>
          <CardDescription class="">{{
            new Date(post.updated).toLocaleString("zh-CN")
          }}</CardDescription>
        </CardHeader>
      </Card>
    </RouterLink>
  </div>
  <template v-else>
    <h1 class="text-4xl font-bold my-4">Posts not found</h1>
    <RouterLink to="/">
      <Badge>Home</Badge>
    </RouterLink>
  </template>
  <Separator class="my-4" />
  <div class="flex justify-end gap-2 flex-wrap">
    <Select v-model="postsPrePage">
      <SelectTrigger class="w-32">
        <LayoutList />
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="20">20</SelectItem>
        <SelectItem value="50">50</SelectItem>
        <SelectItem value="100">100</SelectItem>
      </SelectContent>
    </Select>

    <Pagination
      v-slot="{ page }"
      :total="postsData?.total ?? 0"
      :sibling-count="siblingCount"
      :show-edges="!isSmall"
      :items-per-page="parseInt(postsPrePage)"
      v-model:page="page"
    >
      <PaginationList v-slot="{ items }" class="flex items-center gap-1">
        <PaginationFirst />
        <PaginationPrev />

        <template v-for="(item, index) in items">
          <PaginationListItem
            v-if="item.type === 'page'"
            :key="index"
            :value="item.value"
            as-child
          >
            <Button
              class="w-10 h-10 p-0"
              :variant="item.value === page ? 'default' : 'outline'"
            >
              {{ item.value }}
            </Button>
          </PaginationListItem>
          <PaginationEllipsis v-else :key="item.type" :index="index" />
        </template>

        <PaginationNext />
        <PaginationLast />
      </PaginationList>
    </Pagination>
  </div>
</template>
