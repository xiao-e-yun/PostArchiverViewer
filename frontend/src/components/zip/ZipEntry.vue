<script setup lang="ts">
import { inject, ref, shallowRef, useTemplateRef } from "vue";
import { useIntersectionObserver } from "@vueuse/core";
import { Card } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { zipLoaderKey, type ZipItem } from "./archive";

/** An image in the archive, presented like PostFile; loads near the viewport. */
const props = defineProps<{ item: ZipItem }>();
const emit = defineEmits<{
  open: [item: ZipItem];
  measured: [item: ZipItem, aspect: number];
}>();

// Entries remount when the page swaps the loader, so reading it once is fine.
const loader = inject(zipLoaderKey)!.value;

const src = ref<string>();
const error = shallowRef<unknown>();

async function load() {
  error.value = undefined;
  try {
    src.value = await loader.objectUrl(props.item);
  } catch (e) {
    if (!loader.disposed) error.value = e;
  }
}

const el = useTemplateRef<HTMLElement>("el");
const { stop } = useIntersectionObserver(
  el,
  ([entry]) => {
    if (!entry?.isIntersecting) return;
    stop();
    load();
  },
  { rootMargin: "800px 0px" },
);

/** Once known, size the card like PostFile does from `extra.width/height`. */
const aspect = ref<number>();
function onLoad(event: Event) {
  const img = event.target as HTMLImageElement;
  if (!img.naturalHeight) return;
  aspect.value = img.naturalWidth / img.naturalHeight;
  emit("measured", props.item, aspect.value);
}
</script>

<template>
  <figure ref="el" class="flex flex-col gap-2">
    <Card
      class="m-auto overflow-hidden max-h-[80vh] max-w-full"
      :style="aspect ? { aspectRatio: aspect } : undefined"
    >
      <img
        v-if="src"
        :src="src"
        :alt="item.name"
        :title="item.path"
        class="object-cover max-h-[80vh] w-full h-full cursor-pointer"
        @load="onLoad"
        @click="emit('open', item)"
      />
      <Skeleton v-else class="w-[64vh] max-w-full aspect-[4/5] rounded-none" />
    </Card>

    <div v-if="error" class="text-sm text-destructive text-center">
      Failed to read {{ item.name }}.
      <button class="underline" @click="load">Retry</button>
    </div>
  </figure>
</template>
