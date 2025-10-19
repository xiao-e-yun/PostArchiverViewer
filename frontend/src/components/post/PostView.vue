<script setup lang="ts">
import PostContent from "./PostContent.vue";
import PostHeader from "./PostHeader.vue";
import { provide, toRef } from "vue";
import { postKey } from "./utils";
import type { PostResponse } from "@api/PostResponse";
import type { WithRelations } from "@api/WithRelations";
import { useRelations } from "@/utils";
import PageTitle from "../utils/PageTitle.vue";

const props = defineProps<{
  post: WithRelations<PostResponse> | null;
}>();
const post = toRef(props, "post");

provide(postKey, {
  post,
  relations: useRelations(post),
});
</script>

<template>
  <PageTitle v-if="post"> {{ post.title }} </PageTitle>
  <PostHeader />
  <PostContent />
</template>
