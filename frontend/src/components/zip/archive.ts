import {
  BlobWriter,
  configure,
  HttpReader,
  ZipReader,
  type Entry,
  type FileEntry,
} from "@zip.js/zip.js";
import type { InjectionKey, ShallowRef } from "vue";

// Decompression runs through the native DecompressionStream; workers would
// only add a worker script to ship for no real gain on files this size.
configure({ useWebWorkers: false });

export interface ZipItem {
  /** Position in the sorted listing; stable for the archive, used as a key. */
  index: number;
  path: string;
  name: string;
  /** Parent folder ("" for the root), without trailing slash. */
  dir: string;
  encrypted: boolean;
  /** Set for images, which are shown inline. */
  imageType?: string;
  entry: FileEntry;
}

export interface ZipGroup {
  dir: string;
  items: ZipItem[];
}

export interface ZipListing {
  items: ZipItem[];
  groups: ZipGroup[];
  /** Size of the archive itself, in bytes. */
  size: number;
}

const IMAGE_TYPES: Record<string, string> = {
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  png: "image/png",
  gif: "image/gif",
  webp: "image/webp",
  avif: "image/avif",
  bmp: "image/bmp",
  svg: "image/svg+xml",
};

const JUNK = /(^|\/)(__MACOSX\/|\.DS_Store$|Thumbs\.db$|desktop\.ini$)/i;

// Names without the UTF-8 flag that are not valid UTF-8 either: zip.js would
// read them as CP437. In this archive they are almost always Shift_JIS.
const LEGACY_ENCODINGS = ["shift_jis", "gbk", "big5"];
function decodeLegacyName(value: Uint8Array, encoding: string) {
  if (encoding !== "cp437" || value.every((b) => b < 0x80)) return undefined;
  for (const label of LEGACY_ENCODINGS) {
    try {
      return new TextDecoder(label, { fatal: true }).decode(value);
    } catch {
      // try the next encoding
    }
  }
  return undefined;
}

const collator = new Intl.Collator(undefined, {
  numeric: true,
  sensitivity: "base",
});

function toItem(entry: FileEntry): Omit<ZipItem, "index"> {
  const path = entry.filename;
  const slash = path.lastIndexOf("/");
  const name = path.slice(slash + 1);
  const dot = name.lastIndexOf(".");
  const ext = dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
  return {
    path,
    name,
    dir: slash < 0 ? "" : path.slice(0, slash),
    encrypted: entry.encrypted,
    imageType: IMAGE_TYPES[ext],
    entry,
  };
}

/** Reads only the central directory: a few KB of traffic. */
export async function getZipListing(url: string): Promise<ZipListing> {
  // The file server is usually another origin (the static proxy), where
  // `Content-Range` is unreadable unless exposed and a suffix range needs a
  // CORS preflight. So: HEAD for the size, then plain `bytes=a-b` ranges,
  // which are CORS-safelisted. `preventHeadRequest: false` matters: without
  // it zip.js falls back to downloading the whole archive.
  const http = new HttpReader(url, {
    forceRangeRequests: true,
    preventHeadRequest: false,
  });
  const reader = new ZipReader(http, { decodeText: decodeLegacyName });
  const entries: Entry[] = await reader.getEntries();

  // Folders are derived from file paths: many archives have no directory
  // entries at all.
  const files = entries
    .filter((e): e is FileEntry => !e.directory && !JUNK.test(e.filename))
    .map(toItem)
    .sort(
      (a, b) =>
        Number(a.dir !== "") - Number(b.dir !== "") ||
        collator.compare(a.dir, b.dir) ||
        collator.compare(a.name, b.name),
    );

  const items: ZipItem[] = files.map((f, index) => ({ ...f, index }));
  const groups: ZipGroup[] = [];
  for (const item of items) {
    const last = groups[groups.length - 1];
    if (last && last.dir === item.dir) last.items.push(item);
    else groups.push({ dir: item.dir, items: [item] });
  }
  return { items, groups, size: http.size };
}

/** Shown inline as an image; everything else is a file card. */
export function isGalleryItem(item: ZipItem): boolean {
  return !!item.imageType;
}

/**
 * Loads entry contents for one page visit. Owns every object URL it hands out
 * and revokes them all on dispose, which also aborts in-flight reads.
 */
export function createZipLoader(maxConcurrent = 2) {
  const controller = new AbortController();
  const blobs = new Map<number, Promise<Blob>>();
  const objectUrls = new Map<number, Promise<string>>();
  const created: string[] = [];
  let password: string | undefined;

  let active = 0;
  const waiting: (() => void)[] = [];
  async function acquire(priority: boolean) {
    if (active < maxConcurrent) {
      active++;
      return;
    }
    await new Promise<void>((resolve) =>
      priority ? waiting.unshift(resolve) : waiting.push(resolve),
    );
  }
  function release() {
    const next = waiting.shift();
    if (next) next();
    else active--;
  }

  function blob(item: ZipItem, { priority = false } = {}): Promise<Blob> {
    let result = blobs.get(item.index);
    if (!result) {
      result = (async () => {
        await acquire(priority);
        try {
          controller.signal.throwIfAborted();
          return await item.entry.getData(
            new BlobWriter(item.imageType ?? "application/octet-stream"),
            { signal: controller.signal, password },
          );
        } finally {
          release();
        }
      })();
      blobs.set(item.index, result);
      // Let a failed read be retried.
      result.catch(() => blobs.delete(item.index));
    }
    return result;
  }

  function objectUrl(item: ZipItem, options?: { priority?: boolean }) {
    let result = objectUrls.get(item.index);
    if (!result) {
      result = blob(item, options).then((b) => {
        const url = URL.createObjectURL(b);
        if (controller.signal.aborted) URL.revokeObjectURL(url);
        else created.push(url);
        return url;
      });
      objectUrls.set(item.index, result);
      result.catch(() => objectUrls.delete(item.index));
    }
    return result;
  }

  async function download(item: ZipItem) {
    const url = await objectUrl(item, { priority: true });
    const a = document.createElement("a");
    a.href = url;
    a.download = item.name;
    a.click();
  }

  /** Keeps `value` for encrypted entries if it opens `item`; throws if not. */
  async function unlock(item: ZipItem, value: string) {
    await item.entry.getData(new BlobWriter(), {
      password: value,
      checkPasswordOnly: true,
    });
    password = value;
  }

  function dispose() {
    controller.abort();
    created.forEach((url) => URL.revokeObjectURL(url));
    created.length = 0;
  }

  return {
    objectUrl,
    download,
    unlock,
    dispose,
    get disposed() {
      return controller.signal.aborted;
    },
  };
}

export type ZipLoader = ReturnType<typeof createZipLoader>;

export const zipLoaderKey = Symbol("zipLoader") as InjectionKey<
  ShallowRef<ZipLoader>
>;
