<script lang="ts" setup>
import { computed } from "vue";
import { RouterLink } from "vue-router";
import PostView from "@/components/post/PostView.vue";
import { useRouteParams } from "@vueuse/router";
import type { WithRelations } from "@api/WithRelations";
import type { PostResponse } from "@api/PostResponse";
import { useFetchWithCache } from "@/utils";

const id = useRouteParams("id", "0", { transform: Number });
const url = computed(() => `/api/posts/${id.value}`);

const { data: post, statusCode } = useFetchWithCache<
  WithRelations<PostResponse>
>("post", url);
</script>

<template>
  <template v-if="statusCode === 404">
    <h1>Post not found</h1>
    <RouterLink to="/">Home</RouterLink>
  </template>
  <PostView v-else :post="post" />
</template>
