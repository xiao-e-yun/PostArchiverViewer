<script setup lang="ts">
import type { FileMeta } from "@api/FileMeta";
import { Card } from "../ui/card";
import DialogImage from "../DialogImage.vue";
import DialogTrigger from "../ui/dialog/DialogTrigger.vue";
import DynamicImage from "../image/DynamicImage.vue";
import { File, FolderArchive } from "lucide-vue-next";
import FileCard from "./FileCard.vue";
import { getFileMetaPath } from "@/utils";
import { computed, inject, ref } from "vue";
import { postImagesKey } from "./utils";
import { isZipFile } from "../zip/utils";

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

/** Shown as a FileCard (zip or anything else), full width on phones. */
const isAttachment = computed(
  () => !/^(image|video|audio)\//.test(props.file.mime),
);

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
    :class="{ 'max-sm:w-full': isAttachment }"
    :style="getStyleByFileExtra(file.extra)"
  >
    <template v-if="file.mime.startsWith('image/')">
      <svg
        v-if="hasExtra(file.extra)"
        :width="file.extra.width"
        :height="file.extra.height"
      />

      <DialogImage
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
            :style="{
              position: hasExtra(file.extra) ? 'absolute' : 'relative',
            }"
          />
        </DialogTrigger>
      </DialogImage>
    </template>

    <video
      v-else-if="file.mime.startsWith('video/')"
      :style="getStyleByFileExtra(file.extra)"
      :src="getFileMetaPath(file)"
      class="lazy max-h-[80vh]"
      controls
    />

    <audio
      v-else-if="file.mime.startsWith('audio/')"
      :src="getFileMetaPath(file)"
      controls
    />

    <!-- Anything else; an archive also gets its own page -->
    <FileCard
      v-else
      :name="file.filename"
      :href="getFileMetaPath(file)"
      :icon="isZipFile(file) ? FolderArchive : File"
      :to="
        isZipFile(file)
          ? `/posts/${file.post}/files/${encodeURIComponent(file.filename)}`
          : undefined
      "
    />
  </Card>
</template>
