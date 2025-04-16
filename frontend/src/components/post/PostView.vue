<script setup lang="ts">
import type { PostAPI } from "@/api";
import PostContent from "./PostContent.vue";
import PostHeader from "./PostHeader.vue";
import { provide, toRef } from "vue";
import { postKey } from "./utils";

const props = defineProps<{
  post: PostAPI | null;
}>();

provide(postKey, {
  post: toRef(props, "post"),
});
</script>

<template>
  <PostHeader />
  <PostContent />
</template>
