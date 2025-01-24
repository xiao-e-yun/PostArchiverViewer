<script lang="ts" setup>
import Image from '@/components/image/Image.vue';
import { Badge } from '@/components/ui/badge';
import { Card, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import type { AuthorsAPI } from '@/types';
import { ImageOff } from 'lucide-vue-next';
import { ofetch } from 'ofetch';
import { RouterLink } from 'vue-router';

const authors = await ofetch<AuthorsAPI>("/api/authors");
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
    <RouterLink v-for="author in authors" :to="`/author/${author.id}`">
      <Card class="transition-transform hover:scale-105 hover:z-10 relative aspect-video">
        <div class="z-10 relative bg-background/60 w-full h-full flex flex-col justify-between">
          <CardHeader>
            <CardTitle>{{ author.name }}</CardTitle>
            <CardDescription>{{ new Date(author.updated).toLocaleString() }}</CardDescription>
          </CardHeader>
          <CardFooter class="flex gap-2 capitalize">
            <a v-for="link in author.links.slice(0, 2)" :href="link.url" @click.stop>
              <Badge>{{ link.name }}</Badge>
            </a>
            <Badge v-if="author.links.length > 3">...</Badge>
          </CardFooter>
        </div>
        <Image v-if="author.thumb" :src="author.thumb.url" format="webp"
          class="absolute inset-0 object-cover max-h-full w-full rounded-lg" />
        <ImageOff v-else class="absolute inset-0 w-full h-full p-4" :stroke-width="0.5" />
      </Card>
    </RouterLink>
  </div>
</template>