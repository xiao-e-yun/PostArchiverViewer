<script setup lang="ts">
import { ref, watch } from "vue";
import JSZip from "jszip";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { File, ArrowDown, FileText, Image } from "lucide-vue-next";
import ZipFileTreeItem, { type ZipEntry } from "./ZipFileTreeItem.vue";
import { onUnmounted } from "vue";

const opened = defineModel<boolean>("open");

const props = defineProps<{
  src: string;
}>();

const zip = ref<JSZip | null>(null);
const loading = ref(false);
const loadingProgress = ref(0);
const error = ref<string | null>(null);
const fileTree = ref<ZipEntry[]>([]);
const selectedFile = ref<ZipEntry | null>(null);
const previewContent = ref<string | null>(null);
const previewLoading = ref(false);
const expandedFolders = ref<Set<string>>(new Set());

// Build file tree from zip
function buildFileTree(zipInstance: JSZip): ZipEntry[] {
  const root: ZipEntry[] = [];
  const pathMap = new Map<string, ZipEntry>();

  // First pass: create all entries
  zipInstance.forEach((relativePath, zipEntry) => {
    const parts = relativePath.split("/").filter((p) => p);
    const name = parts[parts.length - 1] || relativePath;
    const isDir = zipEntry.dir;

    const entry: ZipEntry = {
      name,
      path: relativePath,
      isDirectory: isDir,
      size: isDir
        ? 0
        : (
            zipEntry as unknown as {
              _data?: { uncompressedSize?: number };
            }
          )._data?.uncompressedSize || 0,
      children: isDir ? [] : undefined,
    };

    pathMap.set(relativePath, entry);

    // Find parent and add to it
    if (parts.length === 1) {
      root.push(entry);
    } else {
      const parentPath = parts.slice(0, -1).join("/") + "/";
      const parent = pathMap.get(parentPath);
      if (parent && parent.children) {
        parent.children.push(entry);
      } else {
        root.push(entry);
      }
    }
  });

  // Sort: directories first, then alphabetically
  const sortEntries = (entries: ZipEntry[]) => {
    entries.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) {
        return a.isDirectory ? -1 : 1;
      }
      return a.name.localeCompare(b.name);
    });
    entries.forEach((e) => {
      if (e.children) sortEntries(e.children);
    });
  };
  sortEntries(root);

  return root;
}

// Load zip from URL with progress tracking
let abortController: null | AbortController = null;
async function loadZipFromUrl(url: string) {
  loading.value = true;
  loadingProgress.value = 0;
  error.value = null;
  selectedFile.value = null;
  previewContent.value = null;
  expandedFolders.value = new Set();

  try {
    if (abortController) abortController.abort();
    abortController = new AbortController();
    const response = await fetch(url, {
      signal: abortController.signal,
    });
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    // Get total size for progress tracking
    const contentLength = response.headers.get("content-length");
    const total = contentLength ? parseInt(contentLength, 10) : 0;

    // Stream the response to track progress
    const reader = response.body!.getReader();
    const chunks: Uint8Array[] = [];
    let received = 0;

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      chunks.push(value);
      received += value.length;
      loadingProgress.value = Math.round((received / total) * 100);
    }

    // Combine chunks into a single ArrayBuffer
    const arrayBuffer = new Uint8Array(received);
    let position = 0;
    for (const chunk of chunks) {
      arrayBuffer.set(chunk, position);
      position += chunk.length;
    }

    const zipInstance = await JSZip.loadAsync(arrayBuffer);
    zip.value = zipInstance;
    fileTree.value = buildFileTree(zipInstance);
  } catch (e: unknown) {
    // Fetch was aborted, do nothing
    if ((e as Error)?.name === "AbortError") return;

    error.value =
      "Failed to load zip file. Please ensure it is a valid zip archive.";
    zip.value = null;
    fileTree.value = [];
  }
  loading.value = false;
}

// Toggle folder expansion
function toggleFolder(path: string) {
  const newSet = new Set(expandedFolders.value);
  if (newSet.has(path)) newSet.delete(path);
  else newSet.add(path);
  expandedFolders.value = newSet;
}

// Check if content is viewable
function isImage(filename: string): boolean {
  const ext = filename.toLowerCase().split(".").pop() || "";
  return ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico"].includes(
    ext,
  );
}

function isText(filename: string): boolean {
  const ext = filename.toLowerCase().split(".").pop() || "";
  return [
    "txt",
    "md",
    "json",
    "xml",
    "html",
    "css",
    "js",
    "ts",
    "jsx",
    "tsx",
    "vue",
    "py",
    "rb",
    "rs",
    "go",
    "java",
    "c",
    "cpp",
    "h",
    "hpp",
    "yaml",
    "yml",
    "toml",
    "ini",
    "cfg",
    "conf",
    "sh",
    "bash",
    "zsh",
    "log",
    "csv",
    "sql",
    "graphql",
    "dockerfile",
    "makefile",
    "readme",
  ].includes(ext);
}

// Select and preview file
async function selectFile(entry: ZipEntry) {
  if (entry.isDirectory) {
    // Directories are handled by toggle event
    return;
  }

  selectedFile.value = entry;
  previewContent.value = null;

  if (!zip.value) return;

  const zipFile = zip.value.file(entry.path);
  if (!zipFile) return;

  if (isImage(entry.name)) {
    previewLoading.value = true;
    try {
      const blob = await zipFile.async("blob");
      previewContent.value = URL.createObjectURL(blob);
    } catch {
      previewContent.value = null;
    } finally {
      previewLoading.value = false;
    }
  } else if (isText(entry.name)) {
    previewLoading.value = true;
    try {
      previewContent.value = await zipFile.async("string");
    } catch {
      previewContent.value = null;
    } finally {
      previewLoading.value = false;
    }
  }
}

// Download file
async function downloadFile(entry: ZipEntry) {
  if (!zip.value || entry.isDirectory) return;

  const zipFile = zip.value.file(entry.path);
  if (!zipFile) return;

  try {
    const blob = await zipFile.async("blob");
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = entry.name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  } catch {
    // Failed to download
  }
}

// Format file size
function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

// Load zip when dialog opens
watch(opened, (open) => {
  if (open && props.src) {
    loadZipFromUrl(props.src);
  } else if (!open) {
    // Cleanup object URLs
    if (previewContent.value && isImage(selectedFile.value?.name || "")) {
      URL.revokeObjectURL(previewContent.value);
    }
    zip.value = null;
    fileTree.value = [];
    selectedFile.value = null;
    previewContent.value = null;
    error.value = null;
    expandedFolders.value = new Set();
  }
});

onUnmounted(() => {
  if (abortController) abortController.abort();
  if (previewContent.value && isImage(selectedFile.value?.name || ""))
    URL.revokeObjectURL(previewContent.value);
});
</script>

<template>
  <Dialog v-model:open="opened">
    <DialogContent class="max-w-5xl w-[90vw] h-[80vh] flex flex-col p-0 gap-0">
      <DialogHeader class="sr-only">
        <DialogTitle>Zip File Viewer</DialogTitle>
        <DialogDescription>
          Browse and preview files within the zip archive.
        </DialogDescription>
      </DialogHeader>

      <div class="flex-1 flex overflow-hidden">
        <!-- Left: File Browser -->
        <div class="w-1/3 border-r flex flex-col overflow-hidden">
          <div class="flex-1 overflow-auto p-2">
            <!-- Loading skeletons -->
            <div v-if="loading" class="flex flex-col gap-2 p-2">
              <Skeleton class="h-6 w-full" />
              <Skeleton class="h-6 w-3/4 ml-4" />
              <Skeleton class="h-6 w-2/3 ml-4" />
              <Skeleton class="h-6 w-full" />
              <Skeleton class="h-6 w-4/5" />
            </div>

            <!-- File tree -->
            <template v-else>
              <ZipFileTreeItem
                v-for="entry in fileTree"
                :key="entry.path"
                :entry="entry"
                :depth="0"
                :expanded-folders="expandedFolders"
                :selected-path="selectedFile?.path"
                @select="selectFile"
                @toggle="toggleFolder"
              />
            </template>
          </div>
        </div>

        <!-- Right: Preview Pane -->
        <div class="w-2/3 flex flex-col overflow-hidden">
          <!-- Loading state with progress bar -->
          <div
            v-if="loading"
            class="flex flex-col gap-4 p-4 h-full justify-center"
          >
            <div class="text-sm text-muted-foreground text-center">
              Loading zip file...
            </div>
            <div class="w-full bg-muted rounded-full h-2.5 overflow-hidden">
              <div
                class="bg-primary h-2.5 rounded-full transition-all"
                :style="{ width: `${loadingProgress}%` }"
              />
            </div>
          </div>

          <!-- Error state -->
          <div v-else-if="error" class="p-4 text-destructive text-sm">
            {{ error }}
          </div>

          <div
            v-else-if="!selectedFile"
            class="flex-1 flex items-center justify-center text-muted-foreground"
          >
            <div class="text-center">
              <FileText class="w-16 h-16 mx-auto mb-4 opacity-30" />
              <p>Select a file to preview</p>
            </div>
          </div>

          <template v-else>
            <!-- Preview content -->
            <div class="flex-1 overflow-auto">
              <!-- Loading preview -->
              <div
                v-if="previewLoading"
                class="flex items-center justify-center h-full"
              >
                <Skeleton class="w-32 h-32" />
              </div>

              <!-- Image preview -->
              <div
                v-else-if="isImage(selectedFile.name) && previewContent"
                class="flex items-center justify-center p-4 h-full"
              >
                <img
                  :src="previewContent"
                  :alt="selectedFile.name"
                  class="max-w-full max-h-full object-contain"
                />
              </div>

              <!-- Text preview -->
              <div
                v-else-if="isText(selectedFile.name) && previewContent"
                class="p-4 h-full"
              >
                <pre
                  class="text-sm font-mono whitespace-pre-wrap break-words bg-muted p-4 rounded-md overflow-auto max-h-full"
                  >{{ previewContent }}</pre
                >
              </div>

              <!-- No preview available -->
              <div
                v-else
                class="flex-1 flex items-center justify-center h-full"
              >
                <div class="text-center text-muted-foreground">
                  <File class="w-16 h-16 mx-auto mb-4 opacity-30" />
                  <p class="mb-4">Preview not available for this file type</p>
                  <Button variant="outline" @click="downloadFile(selectedFile)">
                    <ArrowDown class="w-4 h-4 mr-2" />
                    Download File
                  </Button>
                </div>
              </div>
            </div>
            <!-- Preview footer -->
            <div
              class="p-3 border-t flex items-center justify-between shrink-0 gap-2"
            >
              <div class="flex items-center gap-2 min-w-0">
                <Image
                  v-if="isImage(selectedFile.name)"
                  class="w-4 h-4 shrink-0"
                />
                <FileText
                  v-else-if="isText(selectedFile.name)"
                  class="w-4 h-4 shrink-0"
                />
                <File v-else class="w-4 h-4 shrink-0" />
                <span class="truncate text-sm font-medium">{{
                  selectedFile.name
                }}</span>
                <span class="text-xs text-muted-foreground shrink-0">
                  ({{ formatSize(selectedFile.size) }})
                </span>
              </div>
              <Button
                size="sm"
                variant="outline"
                class="shrink-0 gap-1"
                @click="downloadFile(selectedFile)"
              >
                <ArrowDown class="w-4 h-4" />
                <span class="hidden sm:inline">Download</span>
              </Button>
            </div>
          </template>
        </div>
      </div>
    </DialogContent>
  </Dialog>
</template>
