<script lang="ts" setup>
import PostList from "@/components/PostList.vue";
import SearchInput from "@/components/SearchInput.vue";
import { useRouteQuery } from "@vueuse/router";
import { computed } from "vue";

const asArray = <T,>(v: T | T[]) => (Array.isArray(v) ? v : [v]);
const transform = <T,>(v: T | T[]) => asArray(v).map(Number);

const rawQuerys = {
  search: useRouteQuery<string>("search", ""),
  platforms: useRouteQuery("platforms", [], { transform }),
  authors: useRouteQuery("authors", [], { transform }),
  tags: useRouteQuery("tags", [], { transform }),
};

const querys = computed({
  get: () => ({
    search: rawQuerys.search.value,
    platforms: rawQuerys.platforms.value,
    authors: rawQuerys.authors.value,
    tags: rawQuerys.tags.value,
  }),
  set: (value) => {
    rawQuerys.search.value = value.search;
    rawQuerys.platforms.value = value.platforms;
    rawQuerys.authors.value = value.authors;
    rawQuerys.tags.value = value.tags;
  },
});
</script>

<template>
  <div>
    <h1 class="text-4xl mb-4">Search</h1>
    <SearchInput v-model="querys" class="mb-4" />
    <PostList url="/api/posts" :query="querys" />
  </div>
</template>
