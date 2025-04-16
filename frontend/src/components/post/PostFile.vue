<script setup lang="ts">
import type { FileMeta } from "@api/FileMeta";
import { Card } from "../ui/card";
import DialogImage from "../DialogImage.vue";
import DialogTrigger from "../ui/dialog/DialogTrigger.vue";
import DynamicImage from "../image/DynamicImage.vue";
import { Badge } from "../ui/badge";
import { ArrowDown, File } from "lucide-vue-next";
import { getFileMetaPath } from "@/utils";

defineProps<{
  file: FileMeta;
}>();

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

//TODO: implement comment
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
      v-if="file.mime.startsWith('image')"
      :aspect="getStyleByFileExtra(file.extra).aspectRatio"
      :src="getFileMetaPath(file)"
      class="p-0"
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
      v-else-if="file.mime.startsWith('video')"
      :src="getFileMetaPath(file)"
      class="lazy max-h-[80vh]"
      controls
    />

    <audio
      v-else-if="file.mime.startsWith('audio')"
      :src="getFileMetaPath(file)"
      controls
    />

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
