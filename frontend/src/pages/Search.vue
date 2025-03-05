<script lang="ts" setup>
import PostList from "@/components/post-list/PostList.vue";
import { postListRestoreKey } from "@/components/post-list/utils";
import SearchInput from "@/components/search/SearchInput.vue";
import { searchRestoreKey } from "@/components/search/utils";
import type { UrlParams } from "@/utils";
import { useEventBus } from "@vueuse/core";
import { omitBy } from "lodash";
import { onMounted, onUnmounted, shallowRef } from "vue";
import { useRouter } from "vue-router";

const querys = shallowRef<UrlParams>();

const router = useRouter();
function update(searchQuerys: UrlParams) {
  querys.value = searchQuerys;
  let query = { ...router.currentRoute.value.query, ...querys.value };
  query = omitBy(query, (query) => query === "");
  delete query.page;

  router.push({ query: query as unknown as Record<string, string> });
  useEventBus(postListRestoreKey).emit(1);
}

const bus = useEventBus(searchRestoreKey);
const restore = () => {
  const query = router.currentRoute.value.query as UrlParams;
  const search = (Array.isArray(query.search) ? query.search : [query.search])
    .filter((v) => v !== null)
    .join(" ");
  const tags = (Array.isArray(query.tags) ? query.tags : [query.tags])
    .map((value) => parseInt(value as string))
    .filter((v) => !isNaN(v));
  querys.value = { search, tags };
  bus.emit(querys.value);
};

onMounted(() => {
  restore();
  addEventListener("popstate", restore);
});
onUnmounted(() => {
  removeEventListener("popstate", restore);
});
</script>

<template>
  <div>
    <h1 class="text-4xl mb-4">Search</h1>
    <SearchInput class="w-full pb-4" @search="update" />
    <PostList url="/api/search" :query="querys ?? {}" />
  </div>
</template>
