import { toRefs, toValue, type MaybeRefOrGetter } from "vue";
import type { FileMeta } from "@api/FileMeta";
import type { WithRelations } from "@api/WithRelations";
import { usePublicConfig } from "./api";
import {
  reactiveComputed,
  useFetch,
  useMemoize,
  useSessionStorage,
  type UseFetchReturn,
} from "@vueuse/core";
import { LRUMap } from "lru_map";

export function useRelations<T>(
  data: MaybeRefOrGetter<WithRelations<T> | null | undefined>,
) {
  return reactiveComputed(() => {
    const relations = toValue(data) ?? ({} as WithRelations<T>);

    const toMap = <T extends { id: number }>(values?: T[]) =>
      new Map(values ? values.map((v) => [v.id, v]) : []);
    const maps = {
      authors: toMap(relations.authors),
      collections: toMap(relations.collections),
      platforms: toMap(relations.platforms),
      tags: toMap(relations.tags),
      fileMetas: toMap(relations.file_metas),
    };

    return {
      ...maps,
      fileMetaPath(id: number): string | undefined {
        const fileMeta = maps.fileMetas.get(id);
        return fileMeta && getFileMetaPath(fileMeta);
      },
    };
  });
}

export function getFileMetaPath(fileMeta: FileMeta) {
  const config = usePublicConfig();
  const url = fileMeta.mime.startsWith("image/")
    ? (config.images_url ?? "/images")
    : (config.resource_url ?? "/resource");
  return (
    url.replace(/\/$/, "") +
    `/${Math.floor(fileMeta.post / 2048)}/${fileMeta.post % 2048}/${fileMeta.filename}`
  );
}

export function getUrl(url: string | URL): URL {
  return new URL(url, location.origin);
}

type UrlBaseParam = string | number | undefined;
export type UrlParams = Record<
  string,
  MaybeRefOrGetter<UrlBaseParam | UrlBaseParam[]>
>;
export function getUrlWithParams(url: string | URL, params: UrlParams): URL {
  const urlObj = getUrl(url);
  Object.entries(params).forEach(([key, rawValue]) => {
    const value = toValue(rawValue);
    if (value === undefined) return;
    const values = Array.isArray(value) ? value : [value];
    for (const value of values) {
      if (value === "") continue;
      urlObj.searchParams.append(key, String(value));
    }
  });
  return urlObj;
}

export function urlParamIntoString(param: string[] | string): string;
export function urlParamIntoString(param: undefined): undefined;
export function urlParamIntoString(param: string[] | string | undefined) {
  return Array.isArray(param) ? param[0] : param;
}

const sessionStorageWithLRU: Record<string, LRUMap<string, unknown>> = {};
export const useSessionStorageWithLRU = <Value>(
  name: string,
  limit: number,
) => {
  if (!sessionStorageWithLRU[name]) {
    sessionStorageWithLRU[name] = useSessionStorage(
      name,
      new LRUMap<string, Value>(limit),
      {
        serializer: {
          read: (raw) => new LRUMap<string, Value>(limit, JSON.parse(raw)),
          write: (value) =>
            JSON.stringify(
              value.toJSON().map(({ key, value }) => [key, value]),
            ),
        },
      },
    ).value;
  }
  return sessionStorageWithLRU[name] as LRUMap<string, Value>;
};

export const useFetchWithCache = <T>(
  category: string,
  url: MaybeRefOrGetter<string>,
  limit = 64,
) => {
  const cache = useSessionStorageWithLRU<UseFetchReturn<T>>(
    "fetch." + category,
    limit,
  );
  const fetch = useMemoize((url: string) => useFetch(url).json<T>(), { cache });
  return toRefs(reactiveComputed(() => fetch(toValue(url))));
};
