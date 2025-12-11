<script setup lang="ts">
import { computed, ref } from "vue";
import type { FileMeta } from "@api/FileMeta";
import type { ArchiveEntry } from "@api/ArchiveEntry";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "../ui/dialog";
import DialogImage from "../DialogImage.vue";
import { FileIcon, DownloadIcon, ImageIcon } from "lucide-vue-next";
import { useFetch } from "@vueuse/core";

const props = defineProps<{
  file: FileMeta;
}>();

const opened = ref(false);

// Fetch archive contents when dialog is opened
const archiveUrl = computed(() => `/api/archive/${props.file.id}`);
const { data: archiveData, execute } = useFetch(archiveUrl, {
  immediate: false,
}).json<{ entries: ArchiveEntry[] }>();

const entries = computed(() => archiveData.value?.entries ?? []);

function onOpenChange(isOpen: boolean) {
  opened.value = isOpen;
  if (isOpen && !archiveData.value) {
    execute();
  }
}

function getExtractUrl(entryName: string) {
  return `/api/archive/${props.file.id}/extract?file=${encodeURIComponent(entryName)}`;
}

function formatFileSize(bytes: number) {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

const selectedImage = ref<string | null>(null);
const selectedImageName = ref<string>("");
const imageDialogOpened = ref(false);

function viewImage(entry: ArchiveEntry) {
  selectedImage.value = getExtractUrl(entry.name);
  selectedImageName.value = entry.name;
  imageDialogOpened.value = true;
}
</script>

<template>
  <Dialog :open="opened" @update:open="onOpenChange">
    <DialogTrigger as-child>
      <slot />
    </DialogTrigger>
    <DialogContent class="max-w-2xl max-h-[80vh] flex flex-col">
      <DialogHeader>
        <DialogTitle>Archive Contents: {{ file.filename }}</DialogTitle>
      </DialogHeader>
      <div class="flex-1 overflow-y-auto pr-4">
        <div
          v-if="entries.length === 0"
          class="text-center py-8 text-muted-foreground"
        >
          Loading archive contents...
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="entry in entries"
            :key="entry.name"
            class="flex items-center gap-3 p-3 rounded-lg border hover:bg-accent transition-colors"
          >
            <div class="flex-shrink-0">
              <ImageIcon v-if="entry.is_image" class="w-5 h-5 text-blue-500" />
              <FileIcon v-else class="w-5 h-5 text-gray-500" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="font-medium truncate">{{ entry.name }}</div>
              <div class="text-sm text-muted-foreground">
                {{ formatFileSize(Number(entry.size)) }}
              </div>
            </div>
            <div class="flex gap-2">
              <button
                v-if="entry.is_image"
                class="p-2 rounded hover:bg-secondary"
                title="View image"
                @click="viewImage(entry)"
              >
                <ImageIcon class="w-4 h-4" />
              </button>
              <a
                :href="getExtractUrl(entry.name)"
                :download="entry.name"
                class="p-2 rounded hover:bg-secondary"
                title="Download file"
              >
                <DownloadIcon class="w-4 h-4" />
              </a>
            </div>
          </div>
        </div>
      </div>
    </DialogContent>
  </Dialog>

  <!-- Image viewer dialog -->
  <DialogImage
    v-if="selectedImage"
    v-model:opened="imageDialogOpened"
    :src="selectedImage"
  >
  </DialogImage>
</template>
