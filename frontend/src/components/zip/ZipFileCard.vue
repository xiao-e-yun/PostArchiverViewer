<script setup lang="ts">
import { computed, inject, ref } from "vue";
import { ArrowDown, File } from "lucide-vue-next";
import { Card } from "@/components/ui/card";
import FileCard from "@/components/post/FileCard.vue";
import { zipLoaderKey, type ZipItem } from "./archive";

/** A non-image file in the archive, as the post's file card: click to download. */
const props = defineProps<{ item: ZipItem }>();

// Entries remount when the page swaps the loader, so reading it once is fine.
const loader = inject(zipLoaderKey)!.value;

const downloading = ref(false);
const failed = ref(false);
async function download() {
  if (downloading.value) return;
  downloading.value = true;
  failed.value = false;
  try {
    await loader.download(props.item);
  } catch {
    if (!loader.disposed) failed.value = true;
  } finally {
    downloading.value = false;
  }
}

const detail = computed(() => (failed.value ? "Failed, try again" : undefined));
</script>

<template>
  <Card class="m-auto overflow-hidden max-w-full max-sm:w-full">
    <FileCard :name="item.name" :icon="File" :detail="detail" @open="download">
      <template #actions>
        <button
          class="inline-flex items-center gap-1 rounded-md border bg-background px-2 sm:px-3 h-8 hover:bg-accent disabled:opacity-50"
          title="Download"
          :disabled="downloading"
          @click.stop="download"
        >
          <ArrowDown class="w-4 h-4" />
          <span class="max-sm:sr-only">Download</span>
        </button>
      </template>
    </FileCard>
  </Card>
</template>
