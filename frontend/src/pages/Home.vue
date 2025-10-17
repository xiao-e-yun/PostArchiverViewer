<script lang="ts" setup>
import {
  ChevronRight,
  Folders,
  Newspaper,
  Quote,
  Tags,
  UsersRound,
} from "lucide-vue-next";
import { RouterLink } from "vue-router";
import { useLazyLoad } from "@/lazyload";
import { useTemplateRef, watch } from "vue";
import PostList from "@/components/PostList.vue";
import { Separator } from "@/components/ui/separator";
import CategoryList from "@/components/CategoryList.vue";

const authorsEl = useTemplateRef<HTMLDialogElement>("authorsList");
watch(authorsEl, (el) => el && useLazyLoad().update());

const RANDOM_QUERY = { order_by: "random" };
</script>

<template>
  <RouterLink to="/authors" class="block mb-8">
    <div class="flex justify-between">
      <h1 class="text-base lg:text-2xl">
        <UsersRound :size="30" class="inline-block mr-2" />
        Authors
      </h1>

      <ChevronRight :size="30" />
    </div>
    <Separator class="my-2" />
    <CategoryList
      category="authors"
      :controls="false"
      :limit="5"
      inline
      :querys="RANDOM_QUERY"
    />
  </RouterLink>

  <RouterLink to="/posts" class="block mb-8">
    <div class="flex justify-between">
      <h1 class="text-base lg:text-2xl">
        <Newspaper :size="30" class="inline-block mr-2" />
        Posts
      </h1>

      <ChevronRight :size="30" />
    </div>
    <Separator class="my-2" />
    <PostList url="/api/posts" :controls="false" :limit="5" inline />
  </RouterLink>

  <RouterLink to="/tags" class="block mb-8">
    <div class="flex justify-between">
      <h1 class="text-base lg:text-2xl">
        <Tags :size="30" class="inline-block mr-2" />
        Tags
      </h1>

      <ChevronRight :size="30" />
    </div>
    <Separator class="my-2" />
    <CategoryList
      category="tags"
      :controls="false"
      :limit="20"
      inline
      :querys="RANDOM_QUERY"
    />
  </RouterLink>

  <RouterLink to="/collections" class="block mb-8">
    <div class="flex justify-between">
      <h1 class="text-base lg:text-2xl">
        <Folders :size="30" class="inline-block mr-2" />
        Collections
      </h1>

      <ChevronRight :size="30" />
    </div>
    <Separator class="my-2" />
    <CategoryList
      category="collections"
      :controls="false"
      :limit="5"
      inline
      :querys="RANDOM_QUERY"
    />
  </RouterLink>

  <RouterLink to="/platforms" class="block mb-12">
    <div class="flex justify-between">
      <h1 class="text-base lg:text-2xl">
        <Quote :size="30" class="inline-block mr-2" />
        Platforms
      </h1>

      <ChevronRight :size="30" />
    </div>
    <Separator class="my-2" />
    <CategoryList category="platforms" :controls="false" :limit="12" inline />
  </RouterLink>
</template>
