<script setup lang="ts">
import { computed } from "vue";
import { File, Folder, FolderOpen } from "lucide-vue-next";

export interface ZipEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  size: number;
  children?: ZipEntry[];
}

const props = defineProps<{
  entry: ZipEntry;
  depth: number;
  expandedFolders: Set<string>;
  selectedPath?: string;
}>();

const emit = defineEmits<{
  select: [entry: ZipEntry];
  toggle: [path: string];
}>();

const isExpanded = computed(() => props.expandedFolders.has(props.entry.path));
const isSelected = computed(() => props.selectedPath === props.entry.path);

function handleClick() {
  if (props.entry.isDirectory) emit("toggle", props.entry.path);
  emit("select", props.entry);
}

const handleChildSelect = (entry: ZipEntry) => emit("select", entry);
const handleChildToggle = (path: string) => emit("toggle", path);
</script>

<template>
  <div>
    <div
      :class="[
        'flex items-center gap-2 px-2 py-1 rounded cursor-pointer text-sm',
        'hover:bg-accent hover:text-accent-foreground',
        isSelected ? 'bg-accent text-accent-foreground' : '',
      ]"
      :style="{ paddingLeft: depth * 16 + 8 + 'px' }"
      @click="handleClick"
    >
      <FolderOpen
        v-if="entry.isDirectory && isExpanded"
        class="w-4 h-4 shrink-0"
      />
      <Folder v-else-if="entry.isDirectory" class="w-4 h-4 shrink-0" />
      <File v-else class="w-4 h-4 shrink-0" />
      <span class="truncate">{{ entry.name }}</span>
    </div>
    <template v-if="entry.isDirectory && isExpanded && entry.children">
      <ZipFileTreeItem
        v-for="child in entry.children"
        :key="child.path"
        :entry="child"
        :depth="depth + 1"
        :expanded-folders="expandedFolders"
        :selected-path="selectedPath"
        @select="handleChildSelect"
        @toggle="handleChildToggle"
      />
    </template>
  </div>
</template>
