<script lang="ts" setup>
import BackTo from "@/components/utils/BackTo.vue";
import PostList from "@/components/PostList.vue";
import { useSearchQuerys } from "@/components/search/search";
import SearchInput from "@/components/search/SearchInput.vue";
import { Button } from "@/components/ui/button";
import { useSessionStorage, useThrottleFn } from "@vueuse/core";
import { Dices } from "lucide-vue-next";
import { watch } from "vue";
import PageTitle from "@/components/utils/PageTitle.vue";

const generateHash = () => Math.random().toString(36).substring(2, 15);
const hash = useSessionStorage("random.posts", generateHash());

const refresh = useThrottleFn(() => (hash.value = generateHash()), 1000);

const querys = useSearchQuerys();
watch(querys, refresh);
</script>

<template>
  <PageTitle> Random </PageTitle>

  <BackTo />
  <h1 class="text-4xl mb-4">Random</h1>

  <SearchInput v-model="querys" class="mb-4" />
  <Button class="mb-4 w-full" variant="outline" @click="refresh">
    <Dices /> Refresh
  </Button>
  <PostList
    url="/api/posts"
    :controls="false"
    :querys="{ ...querys, order_by: 'random', hash }"
  />
  <Button class="my-4 w-full" variant="outline" @click="refresh">
    <Dices /> Refresh
  </Button>
</template>
