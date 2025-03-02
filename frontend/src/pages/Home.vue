<script lang="ts" setup>
import Image from "@/components/image/DynamicImage.vue";
import { Badge } from "@/components/ui/badge";
import {
  Card,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { AuthorsAPI } from "@/api";
import { useFetch } from "@vueuse/core";
import { ImageOff } from "lucide-vue-next";
import { RouterLink } from "vue-router";
import { Skeleton } from "@/components/ui/skeleton";
import { useLazyLoad } from "@/lazyload";
import { useTemplateRef, watch } from "vue";

const { data: authors, isFetching } =
  useFetch("/api/authors").json<AuthorsAPI>();

const authorsEl = useTemplateRef<HTMLDialogElement>("authorsList");
watch(authorsEl, (el) => el && useLazyLoad().update());
</script>

<template>
  <div
    v-if="isFetching"
    class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
  >
    <Skeleton v-for="i in 8" :key="i" class="aspect-video" />
  </div>
  <div
    v-else
    ref="authorsList"
    class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
  >
    <RouterLink
      v-for="author in authors"
      :key="author.id"
      :to="`/author/${author.id}`"
    >
      <Card
        class="transition-transform hover:scale-105 hover:z-10 relative aspect-video"
      >
        <div
          class="z-10 relative bg-background/60 w-full h-full flex flex-col justify-between"
        >
          <CardHeader>
            <CardTitle>{{ author.name }}</CardTitle>
            <CardDescription>{{
              new Date(author.updated).toLocaleString("zh-CN")
            }}</CardDescription>
          </CardHeader>
          <CardFooter class="flex gap-2 capitalize">
            <a
              v-for="link in author.links.slice(0, 2)"
              :key="link.url"
              :href="link.url"
              @click.stop
            >
              <Badge>{{ link.name }}</Badge>
            </a>
            <Badge v-if="author.links.length > 3">...</Badge>
          </CardFooter>
        </div>
        <Image
          v-if="author.thumb"
          :src="author.thumb.url"
          format="webp"
          class="absolute inset-0 object-cover max-h-full w-full rounded-lg"
        />
        <ImageOff
          v-else
          class="absolute inset-0 w-full h-full p-4"
          :stroke-width="0.5"
        />
      </Card>
    </RouterLink>
  </div>
</template>
