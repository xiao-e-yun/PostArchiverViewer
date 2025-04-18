<script lang="ts" setup>
import type { AuthorsAPI, SummaryAPI } from "@/api";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { reactiveComputed, useFetch } from "@vueuse/core";
import {
  Files,
  GitCommitVertical,
  Newspaper,
  Package,
  Tags,
  Users,
} from "lucide-vue-next";
import { RouterLink } from "vue-router";

const { data: summary } = await useFetch("/api/summary").json<SummaryAPI>();

const { data: authors } = await useFetch("/api/authors").json<AuthorsAPI>();

const totalSummary = reactiveComputed(() => {
  const total = {
    authors: 0,
    posts: 0,
    files: 0,
    tags: 0,
  };

  const $summary = summary.value;
  if (!$summary) return total;

  total.tags = $summary.tags;
  for (const author of Object.values($summary.authors)) {
    total.authors++;
    total.posts += author!.posts;
    total.files += author!.files;
  }

  return total;
});
</script>

<template>
  <TooltipProvider as-child>
    <div v-if="summary" class="flex flex-col-reverse md:flex-row gap-4">
      <main class="flex-1 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <Card v-for="(author, id) in summary.authors" :key="id">
          <RouterLink :to="`/author/${id}`" class="p-4 flex flex-col gap-2">
            <div class="flex justify-between">
              <span class="break-keep text-ellipsis overflow-hidden">{{
                authors?.find((a) => a.id == id)?.name
              }}</span>
              <small class="font-bold">#{{ id }}</small>
            </div>
            <div class="flex gap-2">
              <Tooltip>
                <TooltipTrigger as-child>
                  <Badge variant="secondary" class="gap-1 rounded-lg px-2 py-1">
                    <Newspaper class="w-4 h-4" />
                    {{ author!.posts }}
                  </Badge>
                </TooltipTrigger>
                <TooltipContent>Total Post</TooltipContent>
              </Tooltip>

              <Tooltip>
                <TooltipTrigger as-child>
                  <Badge variant="secondary" class="gap-1 rounded-lg px-2 py-1">
                    <Files class="w-4 h-4" />
                    {{ author!.files }}
                  </Badge>
                </TooltipTrigger>
                <TooltipContent>Total Files</TooltipContent>
              </Tooltip>
            </div>
          </RouterLink>
        </Card>
      </main>
      <aside class="md:w-80 flex flex-col gap-4">
        <Card class="p-4 flex flex-col gap-2">
          <p class="flex gap-2">
            <Users />Total Author<Badge variant="secondary" class="ml-auto">{{
              totalSummary.authors
            }}</Badge>
          </p>
          <p class="flex gap-2">
            <Newspaper />Total Post<Badge variant="secondary" class="ml-auto">{{
              totalSummary.posts
            }}</Badge>
          </p>
          <p class="flex gap-2">
            <Files />Total File<Badge variant="secondary" class="ml-auto">{{
              totalSummary.files
            }}</Badge>
          </p>
          <p class="flex gap-2">
            <Tags />Total Tags<Badge variant="secondary" class="ml-auto">{{
              summary.tags
            }}</Badge>
          </p>
        </Card>
        <Card class="p-4 flex flex-col gap-2">
          <p class="flex gap-2">
            <GitCommitVertical />PostArchiver<Badge
              variant="secondary"
              class="ml-auto"
              >v{{ summary.postArchiverVersion }}</Badge
            >
          </p>
          <p class="flex gap-2">
            <Package />Version<Badge variant="secondary" class="ml-auto"
              >v{{ summary.version }}</Badge
            >
          </p>
        </Card>
      </aside>
    </div>
  </TooltipProvider>
</template>
