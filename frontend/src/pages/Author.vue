<script lang="ts" setup>
import type { AuthorAPI } from "@/api";
import { useFetch } from "@vueuse/core";
import { ChevronLeft } from "lucide-vue-next";
import { useRoute } from "vue-router";
import { computed } from "vue";
import { getUrlWithParams } from "@/utils";
import PostList from "@/components/post-list/PostList.vue";
import AuthorHeader from "@/components/author/AuthorHeader.vue";

const route = useRoute();
const author = computed(() => route.params.author as string | undefined);
const authorUrl = computed(
  () => getUrlWithParams(`/api/author`, { author }).href,
);
const { data, isFetching: pending } = useFetch(authorUrl, {
  refetch: true,
}).json<AuthorAPI>();
</script>

<template>
  <RouterLink to="/" class="flex p-2"> <ChevronLeft /> Home </RouterLink>
  <AuthorHeader :author="data" :pending="pending" />
  <PostList url="/api/posts" :query="{ author }" />
</template>
