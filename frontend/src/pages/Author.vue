<script lang="ts" setup>
import Image from '@/components/image/Image.vue';
import { Badge } from '@/components/ui/badge';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import type { AuthorAPI } from '@/api';
import { useFetch } from '@vueuse/core';
import { ChevronLeft, ImageOff } from 'lucide-vue-next';
import { useRoute } from 'vue-router';
import { computed } from 'vue';

let lastId = "0" as string;
const route = useRoute();
const id = computed(() => route.params.author as string | undefined);
const url = computed(() => `/api/author?author=${parseInt(lastId = id.value ?? lastId)}`);
const { data: author, isFetching  } = useFetch(url, { refetch: true }).json<AuthorAPI>();
</script>

<template>
  <RouterLink to="/" class="flex p-2">
    <ChevronLeft /> Home
  </RouterLink>
  <template v-if="isFetching">
    <h1 class="text-4xl font-bold my-4">Loading...</h1>
  </template>
  <template v-else-if="author">
    <div class="py-8">
      <h1 class="text-6xl md:text-8xl">{{ author.name }}</h1>
      <div v-for="link in author.links" class="capitalize">
        <a :href="link.url">
          <Badge>{{ link.name }}</Badge>
        </a>
      </div>
    </div>
    <Separator class="my-4" />
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <RouterLink v-for="post in author.posts" :to="`/post/${post.id}`">
        <Card class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full overflow-hidden">
          <Image v-if="post.thumb" :src="post.thumb.url" :aspect="1 / 1" :width="30"
            class="aspect-square w-full object-cover" />
          <div v-else class="aspect-square">
            <ImageOff class="w-full h-full p-4" :stroke-width="0.5" />
          </div>
          <CardHeader>
            <CardTitle>{{ post.title }}</CardTitle>
            <CardDescription>{{ new Date(post.updated).toLocaleString() }}</CardDescription>
          </CardHeader>
        </Card>
      </RouterLink>
    </div>
  </template>
  <template v-else>
    <h1 class="text-4xl font-bold my-4">Author not found</h1>
    <RouterLink to="/">
      <Badge>Home</Badge>
    </RouterLink>
  </template>
</template>