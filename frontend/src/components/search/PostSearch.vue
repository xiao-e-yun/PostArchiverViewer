<script lang="ts" setup>
import PostList from "@/components/PostList.vue";
import SearchInput from "./SearchInput.vue";
import { computed } from "vue";
import { mergeWith, union } from "lodash";
import { useSearchQuerys } from "./search";

const p = defineProps<{
  defaults?: {
    collections?: number[];
    platforms?: number[];
    authors?: number[];
    tags?: number[];
  };
}>();

const querys = useSearchQuerys();

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
