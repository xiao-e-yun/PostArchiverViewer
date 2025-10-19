<script lang="ts" setup>
import PostList from "@/components/PostList.vue";
import SearchInput from "./SearchInput.vue";
import { computed } from "vue";
import { mergeWith, union } from "lodash";
import { useSearchQuerys, type SearchQuerys } from "./search";

const props = withDefaults(
  defineProps<{
    defaults?: Partial<SearchQuerys>;
  }>(),
  {
    defaults: () => ({}),
  },
);

const querys = useSearchQuerys();

const mergedQuerys = computed<SearchQuerys>(() =>
  mergeWith({}, querys.value, props.defaults ?? {}, (objValue, srcValue) => {
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
