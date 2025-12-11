<script setup lang="ts">
import {
  Info,
  Moon,
  Package,
  Search,
  Sun,
  Dices,
  FolderArchive,
} from "lucide-vue-next";
import { ref } from "vue";

import { useColorMode } from "@vueuse/core";
import { RouterView } from "vue-router";
import { Button } from "./components/ui/button";
import ZipViewer from "./components/ZipViewer.vue";

const mode = useColorMode();
const zipViewerOpen = ref(false);

const links = [
  { name: "Posts", path: "/posts", icon: Search },
  { name: "Random", path: "/random", icon: Dices },
  { name: "About", path: "/about", icon: Info },
];
</script>

<template>
  <div class="flex min-h-screen w-full flex-col">
    <header
      class="sticky top-0 flex h-16 items-center gap-4 border-b bg-background px-4 md:px-6 z-30"
    >
      <RouterLink to="/" class="flex items-center gap-2">
        <Package class="h-8 w-8" />
        <span class="text-nowrap text-lg hidden md:inline font-semibold"
          >Post Archiver</span
        >
      </RouterLink>
      <div class="flex w-full items-center gap-4 md:ml-auto justify-end">
        <Button variant="outline" class="gap-2" @click="zipViewerOpen = true">
          <FolderArchive class="w-4 h-4" />
          <span class="hidden sm:inline">Open</span>
        </Button>
        <RouterLink
          v-for="{ name, path, icon } in links"
          :key="path"
          :to="path"
        >
          <component :is="icon" />
          <span class="sr-only">{{ name }}</span>
        </RouterLink>
        <Button class="p-2" @click="mode = mode === 'dark' ? 'light' : 'dark'">
          <Sun v-if="mode === 'light'" />
          <Moon v-else />
        </Button>
      </div>
    </header>
    <ZipViewer v-model:open="zipViewerOpen" />
    <main class="md:p-8 p-4">
      <RouterView v-slot="{ Component }">
        <KeepAlive>
          <Suspense>
            <div>
              <component :is="Component" />
            </div>
          </Suspense>
        </KeepAlive>
      </RouterView>
    </main>
  </div>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=Comfortaa:wght@300..700&family=Overpass:wght@500&display=swap");

body {
  font-family: "Comfortaa", sans-serif;
}

/* for lazy loading images */
img:not([src]):not([srcset]) {
  visibility: hidden;
}
</style>
