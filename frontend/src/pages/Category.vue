<script lang="ts" setup>
import BackTo from "@/components/utils/BackTo.vue";
import CategoryList from "@/components/CategoryList.vue";
import PageTitle from "@/components/utils/PageTitle.vue";

defineProps<{
  category: "authors" | "collections" | "tags" | "platforms";
}>();
</script>

<template>
  <PageTitle> {{ category }} </PageTitle>

  <BackTo />
  <h1 class="text-4xl font-bold mb-4 capitalize">
    {{ category }}
  </h1>
  <CategoryList
    :category="category"
    :limit="['tags', 'platforms'].includes(category) ? 100 : undefined"
  />
</template>
