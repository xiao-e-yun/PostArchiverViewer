<script lang="ts" setup>
import type { SummaryResponse } from "@api/SummaryResponse";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useFetch } from "@vueuse/core";
import {
  Files,
  Folders,
  GitCommitVertical,
  Newspaper,
  Package,
  Quote,
  Tags,
  Users,
} from "lucide-vue-next";

const { data: summary } =
  await useFetch("/api/summary").json<SummaryResponse>();
</script>

<template>
  <TooltipProvider as-child>
    <div v-if="summary" class="flex flex-col md:flex-row gap-4">
      <main class="flex-1">
        <h1 class="text-4xl mb-4">Post Archiver Viewer</h1>
        <a
          href="https://github.com/xiao-e-yun/PostArchiverViewer"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Badge variant="secondary" class="mb-4">
            <span class="flex items-center gap-2">
              <GitCommitVertical />View on GitHub
            </span>
          </Badge>
        </a>
      </main>
      <aside class="md:w-80 flex flex-col gap-4">
        <Card class="p-4 flex flex-col gap-2">
          <h1 class="text-xl font-bold mb-2">Summary</h1>
          <p class="flex gap-2">
            <Newspaper />Total Post<Badge variant="secondary" class="ml-auto">{{
              summary.posts
            }}</Badge>
          </p>
          <p class="flex gap-2">
            <Files />Total File<Badge variant="secondary" class="ml-auto">{{
              summary.files
            }}</Badge>
          </p>
          <p class="flex gap-2">
            <Users />Total Author<Badge variant="secondary" class="ml-auto">{{
              summary.authors
            }}</Badge>
          </p>
          <p class="flex gap-2">
            <Tags />Total Tags<Badge variant="secondary" class="ml-auto">{{
              summary.tags
            }}</Badge>
          </p>
          <p class="flex gap-2">
            <Folders />Total Collection<Badge
              variant="secondary"
              class="ml-auto"
              >{{ summary.collections }}</Badge
            >
          </p>
          <p class="flex gap-2">
            <Quote />Total Platform<Badge variant="secondary" class="ml-auto">{{
              summary.platforms
            }}</Badge>
          </p>
        </Card>
        <Card class="p-4 flex flex-col gap-2">
          <h1 class="text-xl font-bold mb-2">Versions</h1>
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
