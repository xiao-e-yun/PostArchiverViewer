<script lang="ts" setup>
import type { PostAPI } from "@/api";
import { computed } from "vue";
import { useFetch } from "@vueuse/core";
import { getUrlWithParams } from "@/utils";
import { RouterLink, useRoute } from "vue-router";
import PostView from "@/components/post/PostView.vue";

const route = useRoute();
const id = computed(() => route.params.post as string | undefined);
const url = computed(() => getUrlWithParams("/api/post", { post: id }).href);

const { data: post, statusCode } = useFetch(url, {
  refetch: true,
}).json<PostAPI>();
</script>

<template>
  <template v-if="statusCode === 404">
    <h1>Post not found</h1>
    <RouterLink to="/">Home</RouterLink>
  </template>
  <PostView v-else :post="post" />
</template>
