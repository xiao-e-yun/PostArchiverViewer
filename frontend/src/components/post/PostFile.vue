<script setup lang="ts">
import type { FileMeta } from "@api/FileMeta";
import { Card } from "../ui/card";
import DialogImage from "../DialogImage.vue";
import DialogTrigger from "../ui/dialog/DialogTrigger.vue";
import DynamicImage from "../image/DynamicImage.vue";
import ArchiveBrowser from "./ArchiveBrowser.vue";
import { Badge } from "../ui/badge";
import { ArrowDown, File, FileArchive } from "lucide-vue-next";
import { getFileMetaPath } from "@/utils";
import { computed, inject, ref } from "vue";
import { postImagesKey } from "./utils";

const props = defineProps<{
  file: FileMeta;
}>();

const images = inject(
  postImagesKey,
  computed(() => []),
);

function getStyleByFileExtra(extra: FileMeta["extra"]) {
  if (!hasExtra(extra)) return {};
  const width = parseInt(extra.width as string);
  const height = parseInt(extra.height as string);
  return {
    aspectRatio: width / height,
  };
}

function hasExtra(extra: FileMeta["extra"]) {
  return extra && (extra.width || extra.height);
}

function getExt(file: FileMeta) {
  return file.filename.slice(file.filename.indexOf("."));
}

function isCompressedFile(file: FileMeta) {
  return (
    file.mime === "application/zip" ||
    file.mime === "application/x-zip-compressed" ||
    file.filename.toLowerCase().endsWith(".zip")
  );
}

const index = ref<FileMeta | null>(null);
function switchImage(prev: boolean) {
  const currentId = index.value ? index.value.id : props.file.id;
  const currentIndex = images.value.findIndex((v) => v.id === currentId);
  const next = images.value[currentIndex + (prev ? -1 : 1)];
  if (!next) return;
  index.value = next;
}

function onClickToSwitch(event: MouseEvent) {
  const position =
    event.offsetX / (event.currentTarget! as HTMLImageElement).clientWidth;
  const prev = position < 0.5;
  switchImage(prev);
}

function resetIndex(opened: boolean) {
  if (!opened) return;
  index.value = null;
}
</script>

<template>
  <Card
    class="m-auto overflow-hidden max-h-[80vh] max-w-full relative"
    :style="getStyleByFileExtra(file.extra)"
  >
    <svg
      v-if="hasExtra(file.extra)"
      :width="file.extra.width"
      :height="file.extra.height"
    />

    <DialogImage
      v-if="file.mime.startsWith('image/')"
      :aspect="getStyleByFileExtra((index || file).extra).aspectRatio"
      :src="getFileMetaPath(index || file)"
      class="p-0"
      @update:opened="resetIndex"
      @click="onClickToSwitch"
    >
      <DialogTrigger as="div">
        <DynamicImage
          :width="100"
          :src="getFileMetaPath(file)"
          :aspect="getStyleByFileExtra(file.extra).aspectRatio"
          class="object-cover max-h-[80vh] w-full h-full inset-0"
          :style="{ position: hasExtra(file.extra) ? 'absolute' : 'relative' }"
        />
      </DialogTrigger>
    </DialogImage>

    <video
      v-else-if="file.mime.startsWith('video/')"
      :src="getFileMetaPath(file)"
      class="lazy max-h-[80vh]"
      controls
    />

    <audio
      v-else-if="file.mime.startsWith('audio/')"
      :src="getFileMetaPath(file)"
      controls
    />

    <!-- Compressed file - show archive browser -->
    <ArchiveBrowser v-else-if="isCompressedFile(file)" :file="file">
      <div
        class="sm:w-72 flex flex-col items-center p-4 gap-2 relative cursor-pointer hover:bg-accent/50 transition-colors rounded"
      >
        <div class="w-full h-full relative">
          <FileArchive class="w-full h-full text-purple-500" />
        </div>
        <Badge class="py-1 px-2">
          <FileArchive class="h-4 mr-1" />
          View Archive
        </Badge>
      </div>
    </ArchiveBrowser>

    <div v-else class="sm:w-72 flex flex-col items-center p-4 gap-2 relative">
      <div class="w-full h-full relative">
        <File class="w-full h-full" />
        <div
          v-for="ext in getExt(file)"
          :key="ext"
          class="absolute inset-y-9 inset-x-16 pt-20 text-center hidden sm:visible"
        >
          <span v-if="ext.length <= 3" class="text-6xl capitalize">
            {{ ext }}
          </span>
          <span
            v-else
            class="block w-full text-4xl capitalize text-ellipsis overflow-hidden"
          >
            {{ ext }}
          </span>
        </div>
      </div>
      <a
        target="_blank"
        :href="getFileMetaPath(file)"
        rel="noopener noreferrer"
      >
        <Badge class="py-1 px-2">
          <ArrowDown class="h-4" />
          {{ file.filename }}
        </Badge>
      </a>
    </div>
  </Card>
</template>
