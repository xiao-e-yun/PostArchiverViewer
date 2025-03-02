<script setup lang="ts">
import type { PostAPI } from "@/api";
import PostContent from "./PostContent.vue";
import PostHeader from "./PostHeader.vue";
import { Separator } from "../ui/separator";
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
  <Separator class="my-4" />
  <PostContent />
</template>
