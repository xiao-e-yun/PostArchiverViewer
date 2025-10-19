<script lang="ts" setup>
import PostList from "@/components/PostList.vue";
import { useSearchQuerys } from "@/components/search/search";
import SearchInput from "@/components/search/SearchInput.vue";
import { Button } from "@/components/ui/button";
import { useSessionStorage, useThrottleFn } from "@vueuse/core";
import { Dices } from "lucide-vue-next";
import { watch } from "vue";

const generateHash = () => Math.random().toString(36).substring(2, 15);
const hash = useSessionStorage("random.posts", generateHash());

const refresh = useThrottleFn(() => (hash.value = generateHash()), 1000);

const querys = useSearchQuerys();
watch(querys, refresh);
</script>

<template>
  <div class="flex flex-col items-center gap-2 mb-8 relative">
    <RouterLink class="mr-auto ml-4 capitalize" to="/random">
      <h1 class="text-4xl">Random</h1>
    </RouterLink>
  </div>

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
