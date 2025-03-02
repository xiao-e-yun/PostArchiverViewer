<script setup lang="ts">
import { computed, provide, ref, watch } from "vue";
import { Separator } from "../ui/separator";
import PostListControl from "./PostListControl.vue";
import { getUrlWithParams, urlParamIntoString } from "@/utils";
import {
  extendRef,
  refThrottled,
  useFetch,
  useLocalStorage,
  useUrlSearchParams,
} from "@vueuse/core";
import type { PostsAPI } from "@/api";
import {
  postListContentKey,
  postListControlKey,
  postsPrePageKey,
} from "./utils";
import PostListContent from "./PostListContent.vue";

const props = defineProps<{
  url: string;
  query: Record<string, string | number | undefined>;
}>();

const postsPrePage = useLocalStorage(postsPrePageKey, 20);

const searchParams = useUrlSearchParams("history", { writeMode: "push" });

const pageIndex = ref(parseInt(urlParamIntoString(searchParams.page) || "1"));
const pageThrottled = refThrottled(pageIndex, 500);
watch(pageThrottled, (value) => {
  if (value === 1) delete searchParams.page;
  else searchParams.page = value.toString();
});

const url = computed(
  () =>
    getUrlWithParams(props.url, {
      page: pageThrottled.value - 1,
      limit: postsPrePage.value,
      ...props.query,
    }).href,
);

const { data, isFetching: pending } = useFetch(url, {
  refetch: true,
}).json<PostsAPI>();
const posts = computed(() => data.value?.posts);
const total = extendRef(ref<number>(), { url: url.value });
watch(data, () => {
  const changed = total.value === undefined;
  if (changed && !data.value) return (total.value = undefined);
  if (data.value) total.value = data.value.total;
});

provide(postListControlKey, {
  postsPrePage,
  pageIndex,
  total,
});

provide(postListContentKey, {
  postsPrePage,
  posts,
});

const errorText = computed(() => {
  if (data.value) return "No posts found.";
  return "Something went wrong.";
});
</script>

<template>
  <div v-if="pending || data?.posts.length">
    <PostListControl />
    <Separator class="my-4" />
    <PostListContent />
    <Separator class="my-4" />
    <PostListControl />
  </div>
  <div v-else class="p-4 h-64">
    <h1 class="text-4xl font-bold my-4">{{ errorText }}</h1>
  </div>
</template>
