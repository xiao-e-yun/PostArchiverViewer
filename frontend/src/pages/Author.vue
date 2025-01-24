<script lang="ts" setup>
import Image from '@/components/image/Image.vue';
import { Badge } from '@/components/ui/badge';
import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import type { AuthorsAPI, PostsAPI } from '@/types';
import { ChevronLeft, ImageOff } from 'lucide-vue-next';
import { ofetch } from 'ofetch';
import { useRoute } from 'vue-router';

const authors = await ofetch<AuthorsAPI>("/api/authors");
const pageAuthorId = parseInt(useRoute().params.id as string);
const author = authors.find(author => author.id === pageAuthorId);

const posts = await ofetch<PostsAPI>("/api/posts", { query: { author: pageAuthorId } });
</script>

<template>
  <RouterLink v-if="author" :to="`/`" class="flex p-2">
    <ChevronLeft /> Home
  </RouterLink>
  <template v-if="author">
    <div class="min-h-[30vh]">
      <h1 class="text-[10vh]">{{ author.name }}</h1>
      <div v-for="link in author.links" class="capitalize">
        <a :href="link.url">
          <Badge>{{ link.name }}</Badge>
        </a>
      </div>
    </div>
    <Separator class="my-4" />
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <RouterLink v-for="post in posts" :to="`/post/${post.id}`">
        <Card class="transition-transform hover:scale-105 hover:z-10 relative w-full h-full overflow-hidden">
          <Image v-if="post.thumb" :src="post.thumb.url" :aspect="1/1" :width="30" class="aspect-square w-full object-cover" />
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
    <h1>Author not found</h1>
    <RouterLink to="/">Home</RouterLink>
  </template>
</template>