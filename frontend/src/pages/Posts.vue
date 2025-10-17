<script lang="ts" setup>
import PostSearch from "@/components/search/PostSearch.vue";
</script>

<template>
  <div class="flex flex-col items-center gap-2 mb-8 relative">
    <RouterLink class="mr-auto ml-4 capitalize" to="/posts">
      <h1 class="text-4xl">Search</h1>
    </RouterLink>
  </div>
  <PostSearch />
</template>
