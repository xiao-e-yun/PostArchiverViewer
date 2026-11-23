<script lang="ts" setup>
import {
  computed,
  onUnmounted,
  provide,
  reactive,
  ref,
  shallowRef,
  watch,
} from "vue";
import { useRouteParams } from "@vueuse/router";
import { ArrowDown, Folder, FolderArchive } from "lucide-vue-next";
import type { WithRelations } from "@api/WithRelations";
import type { PostResponse } from "@api/PostResponse";
import { getFileMetaPath, useFetchWithCache } from "@/utils";
import BackTo from "@/components/utils/BackTo.vue";
import PageTitle from "@/components/utils/PageTitle.vue";
import DialogImage from "@/components/DialogImage.vue";
import { Skeleton } from "@/components/ui/skeleton";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import ZipEntry from "@/components/zip/ZipEntry.vue";
import ZipFileCard from "@/components/zip/ZipFileCard.vue";
import {
  createZipLoader,
  getZipListing,
  isGalleryItem,
  zipLoaderKey,
  type ZipItem,
  type ZipListing,
} from "@/components/zip/archive";
import { formatSize } from "@/components/zip/utils";

const postId = useRouteParams("id", "0", { transform: Number });
const filename = useRouteParams<string>("filename", "");

const { data: post, statusCode } = useFetchWithCache<
  WithRelations<PostResponse>
>(
  "post",
  computed(() => `/api/posts/${postId.value}`),
);

const file = computed(() =>
  post.value?.id === postId.value
    ? post.value.file_metas?.find((f) => f.filename === filename.value)
    : undefined,
);
// A filename is unique within its post, and the path is derived from the two,
// so the listing starts right away instead of waiting for the post.
const src = computed(
  () =>
    filename.value &&
    getFileMetaPath({
      post: postId.value,
      filename: filename.value,
      mime: "application/zip",
    }),
);

// --- Password, for encrypted archives -------------------------------------
const unlocked = ref(false);
const password = ref<string | number>("");
const wrongPassword = ref(false);
const checking = ref(false);
/** Nothing is shown until an encrypted archive is unlocked. */
const requiredPassword = computed(
  () => !unlocked.value && !!listing.value?.items.some((i) => i.encrypted),
);
async function unlock() {
  const item = listing.value?.items.find((i) => i.encrypted);
  if (!item || checking.value) return;
  checking.value = true;
  wrongPassword.value = false;
  try {
    await loader.value.unlock(item, String(password.value));
    unlocked.value = true;
  } catch {
    wrongPassword.value = true;
  } finally {
    checking.value = false;
  }
}

// --- Entry loader, one per archive ----------------------------------------
// It owns the decoded blobs; switching archives (or leaving) releases them.
// Entries unmount while the next listing loads, so they pick up the new one.
const loader = shallowRef(createZipLoader());
const lightboxOpen = ref(false);
const lightboxSrc = ref("");
function renewLoader() {
  loader.value.dispose();
  loader.value = createZipLoader();
  lightboxOpen.value = false;
  lightboxSrc.value = "";
  unlocked.value = false;
  password.value = "";
  wrongPassword.value = false;
}
provide(zipLoaderKey, loader);
onUnmounted(() => loader.value.dispose());

// --- Listing --------------------------------------------------------------
const listing = shallowRef<ZipListing | null>(null);
const listError = ref(false);

watch(
  src,
  async (url, _, onCleanup) => {
    let cancelled = false;
    onCleanup(() => (cancelled = true));
    listing.value = null;
    listError.value = false;
    renewLoader();
    if (!url) return;
    try {
      const result = await getZipListing(url);
      if (cancelled) return;
      listing.value = result;
    } catch {
      if (!cancelled) listError.value = true;
    }
  },
  { immediate: true },
);

/** Per folder (root first): its media inline, then its other files as a list. */
const sections = computed(() =>
  (listing.value?.groups ?? []).map((g) => ({
    dir: g.dir,
    media: g.items.filter(isGalleryItem),
    files: g.items.filter((i) => !isGalleryItem(i)),
  })),
);

// --- Lightbox over the archive's images -----------------------------------
const images = computed(() => listing.value?.items.filter(isGalleryItem) ?? []);
const aspects = reactive(new Map<number, number>());
const lightboxItem = shallowRef<ZipItem>();

async function showImage(item: ZipItem) {
  lightboxItem.value = item;
  try {
    lightboxSrc.value = await loader.value.objectUrl(item, { priority: true });
    lightboxOpen.value = true;
  } catch {
    // The entry shows its own error.
  }
}

function onLightboxClick(event: MouseEvent) {
  const target = event.currentTarget as HTMLElement;
  const prev = event.offsetX / target.clientWidth < 0.5;
  const list = images.value;
  const current = lightboxItem.value ? list.indexOf(lightboxItem.value) : -1;
  const next = list[current + (prev ? -1 : 1)];
  if (next) showImage(next);
}
</script>

<template>
  <PageTitle>{{ filename || "Archive" }}</PageTitle>

  <template v-if="statusCode === 404 || (post && post.id === postId && !file)">
    <BackTo :to="statusCode === 404 ? '/' : `/posts/${postId}`" />
    <h1>File not found</h1>
  </template>

  <template v-else>
    <!-- Same column as the post, but a compact header: this is an attachment -->
    <div class="relative">
      <header class="lg:w-[1024px] mx-auto flex flex-col gap-3 pb-6">
        <BackTo :to="`/posts/${postId}`" class="pb-0 max-w-full">
          <span class="truncate">{{ post?.title ?? "Post" }}</span>
        </BackTo>
        <h1
          class="flex items-center gap-2 md:text-3xl text-2xl font-bold break-all"
          :title="filename"
        >
          <FolderArchive class="w-7 h-7 md:w-8 md:h-8 shrink-0" />
          {{ filename }}
        </h1>

        <div class="flex gap-2 flex-wrap items-center">
          <template v-if="listing">
            <Badge>
              {{ listing.items.length }}
              {{ listing.items.length === 1 ? "file" : "files" }}
            </Badge>
            <Badge class="bg-blue-300 dark:bg-blue-600" variant="secondary">
              {{ formatSize(listing.size) }}
            </Badge>
          </template>
          <Badge v-else-if="listError" variant="destructive">Unreadable</Badge>
          <Skeleton v-else class="rounded-full w-32 h-[22px]" />
          <a v-if="src" :href="src" target="_blank" rel="noopener noreferrer">
            <Badge variant="outline" class="gap-1 bg-background">
              <ArrowDown class="w-3 h-3" /> Download zip
            </Badge>
          </a>
        </div>
      </header>
      <!-- Full-bleed band behind the header, where the post has its blurred thumb -->
      <div
        class="absolute -top-4 md:-top-8 bottom-0 -left-4 md:-left-8 w-[calc(100%+2rem)] md:w-[calc(100%+4rem)] -z-10 bg-muted border-b"
      />
    </div>

    <div class="lg:w-[1024px] mx-auto">
      <div class="py-6 md:border-x md:px-6">
        <div v-if="listError" class="text-destructive">
          The archive could not be read. It may be damaged, or the file server
          may not allow range requests.
        </div>

        <template v-else-if="requiredPassword">
          <div class="flex gap-2 sm:w-sm max-w-full mx-auto py-8">
            <Input
              v-model="password"
              placeholder="Archive password"
              autofocus
              @keydown.enter="unlock"
            />
            <Button :disabled="checking" @click="unlock">Unlock</Button>
          </div>
          <p v-if="wrongPassword" class="text-sm text-destructive text-center">
            Wrong password.
          </p>
        </template>

        <div v-else-if="listing" class="flex flex-col gap-8">
          <p
            v-if="!listing.items.length"
            class="text-muted-foreground text-center"
          >
            This archive is empty.
          </p>

          <section
            v-for="group in sections"
            :key="group.dir"
            class="flex flex-col gap-4"
          >
            <!-- A folder is a heading over its files -->
            <h2
              v-if="group.dir"
              class="flex items-center gap-2 text-lg md:text-xl font-semibold min-w-0"
              :title="group.dir"
            >
              <Folder class="w-5 h-5 md:w-6 md:h-6 shrink-0" />
              <span class="truncate">{{ group.dir }}/</span>
            </h2>
            <ZipEntry
              v-for="item in group.media"
              :key="item.index"
              :item="item"
              @open="showImage"
              @measured="(i, aspect) => aspects.set(i.index, aspect)"
            />
            <ZipFileCard
              v-for="item in group.files"
              :key="item.index"
              :item="item"
            />
          </section>
        </div>

        <div v-else class="flex flex-col gap-4">
          <Skeleton style="aspect-ratio: 0.8; height: 80vh; margin: auto" />
        </div>
      </div>
    </div>

    <DialogImage
      v-model:opened="lightboxOpen"
      :src="lightboxSrc"
      :aspect="lightboxItem && aspects.get(lightboxItem.index)"
      class="p-0"
      @click="onLightboxClick"
    />
  </template>
</template>
