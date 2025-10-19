<script lang="ts" setup>
import BackTo from "@/components/utils/BackTo.vue";
import PostSearch from "@/components/search/PostSearch.vue";
import PageTitle from "@/components/utils/PageTitle.vue";
</script>

<template>
  <PageTitle> Posts </PageTitle>
  <div>
    <BackTo />
    <h1 class="text-4xl mb-4">Posts</h1>
    <PostSearch />
  </div>
</template>
