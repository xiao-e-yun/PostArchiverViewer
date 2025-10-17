<script lang="ts" setup>
import PostList from "@/components/PostList.vue";
import { useRouteQuery } from "@vueuse/router";
import SearchInput from "./SearchInput.vue";
import { computed, toValue } from "vue";
import { forEach, mapValues, mergeWith, union } from "lodash";

const p = defineProps<{
  defaults?: {
    collections?: number[];
    platforms?: number[];
    authors?: number[];
    tags?: number[];
  };
}>();

const asArray = <T,>(v: T | T[]) => (Array.isArray(v) ? v : [v]);
const transform = <T,>(v: T | T[]) => asArray(v).map(Number);

const rawQuerys = {
  search: useRouteQuery<string>("search", ""),
  collections: useRouteQuery("collections", [], { transform }),
  platforms: useRouteQuery("platforms", [], { transform }),
  authors: useRouteQuery("authors", [], { transform }),
  tags: useRouteQuery("tags", [], { transform }),
};

type Querys = {
  search: string;
  collections: number[];
  platforms: number[];
  authors: number[];
  tags: number[];
};

const querys = computed<Querys>({
  set: (values) =>
    forEach(values, (v, k) => (rawQuerys[k as keyof Querys].value = v)),
  get: () => mapValues(rawQuerys, toValue) as unknown as Querys,
});

const mergedQuerys = computed(() =>
  mergeWith({}, querys.value, p.defaults ?? {}, (objValue, srcValue) => {
    if (Array.isArray(objValue)) return union(objValue, srcValue);
  }),
);
</script>

<template>
  <div>
    <SearchInput v-model="querys" class="mb-4" />
    <PostList url="/api/posts" :querys="mergedQuerys" />
  </div>
</template>
