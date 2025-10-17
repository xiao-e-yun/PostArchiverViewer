<script lang="ts" setup>
import Search from "@/components/search/Search.vue";
</script>

<template>
  <div class="flex flex-col items-center gap-2 mb-8 relative">
    <RouterLink class="mr-auto ml-4 capitalize" to="/posts">
      <h1 class="text-4xl">Posts</h1>
    </RouterLink>
  </div>
  <Search />
</template>
