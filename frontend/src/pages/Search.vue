<script lang="ts" setup>
import PostList from "@/components/post-list/PostList.vue";
import { postListRestoreKey } from "@/components/post-list/utils";
import SearchInput from "@/components/search/SearchInput.vue";
import type { UrlParams } from "@/utils";
import { useEventBus } from "@vueuse/core";
import { omitBy } from "lodash";
import { shallowRef, watch } from "vue";
import { useRouter } from "vue-router";

const querys = shallowRef<UrlParams>();

const router = useRouter();
watch(querys, () => {
  let query = { ...router.currentRoute.value.query, ...querys.value };
  query = omitBy(query, (query) => query === "");
  delete query.page;

  router.push({ query: query as unknown as Record<string, string> });
  useEventBus(postListRestoreKey).emit(1);
});
</script>

<template>
  <div>
    <h1 class="text-4xl mb-4">Search</h1>
    <SearchInput class="w-full pb-4" @search="querys = $event" />
    <PostList v-if="querys" url="/api/search" :query="querys" />
  </div>
</template>
