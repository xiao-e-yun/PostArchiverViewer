<script setup lang="ts">
import { inject } from "vue";
import { postListContentKey } from "./utils";
import PostListItem from "./PostListItem.vue";
import { useLazyLoad } from "@/lazyload";
import { Skeleton } from "../ui/skeleton";
import { throttle } from "lodash";

const { posts, postsPrePage } = inject(postListContentKey)!;
const lazyload = useLazyLoad();
const update = throttle(() => lazyload.update(), 50, {
  leading: false,
  trailing: true,
});
</script>

<template>
  <div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-4">
    <template v-if="!posts">
      <Skeleton
        v-for="i in postsPrePage"
        :key="i"
        class="pt-[100%] h-[122px] box-content"
      />
    </template>
    <PostListItem
      v-for="post in posts"
      :key="post.id"
      :post="post"
      @vue:mounted="update"
    />
  </div>
</template>
