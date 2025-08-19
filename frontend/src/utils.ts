import { shallowRef, toRefs, toValue, type MaybeRefOrGetter } from "vue";
import type { FileMeta } from "@api/FileMeta";
import type { WithRelations } from "@api/WithRelations";
import { usePublicConfig } from "./api";
import {
  computedWithControl,
  reactiveComputed,
  toReactive,
  until,
  useFetch,
  useMemoize,
  type UseFetchReturn,
} from "@vueuse/core";
import { LRUMap } from "lru_map";
import { computed } from "vue";

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

const fetchCaches = {} as Record<string, FetchCache<unknown>>;
export const fetchCache = <T>(name: string, limit: number) =>
  (fetchCaches[name] ??= new FetchCache<T>(
    `fetch.${name}`,
    limit,
  )) as FetchCache<T>;

class FetchCache<T> extends LRUMap<string, UseFetchReturn<T>> {
  constructor(
    public sessionName: string,
    limit: number,
  ) {
    const rawData = sessionStorage.getItem(sessionName) ?? "[]";
    const data = (JSON.parse(rawData) as [string, T][]).map(([key, data]) => [
      key,
      {
        // Fake type assertion to match UseFetchReturn<T>
        isFinished: shallowRef(true),
        isFetching: shallowRef(false),
        statusCode: shallowRef(200),
        error: shallowRef(null),
        canAbort: shallowRef(false),
        aborted: shallowRef(false),
        data: shallowRef(data),
      },
    ]) as unknown as [string, UseFetchReturn<T>][];
    super(limit, data);
  }
  set(key: string, value: UseFetchReturn<T>) {
    const result = super.set(key, value);
    this.saveToSession();
    return result;
  }

  delete(key: string) {
    const result = super.delete(key);
    this.saveToSession();
    return result;
  }

  clear() {
    const result = super.clear();
    sessionStorage.removeItem(this.sessionName);
    return result;
  }

  async saveToSession() {
    for (const value of this.values() as unknown as UseFetchReturn<T>[])
      await until(value.isFinished).toBe(true);

    const data = super
      .toJSON()
      .filter(({ value }) => !!value.data.value)
      .map(({ key, value }) => [key, value.data.value]) as [string, T][];
    sessionStorage.setItem(this.sessionName, JSON.stringify(data));
  }
}

export const useFetchWithCache = <T>(
  category: string,
  url: MaybeRefOrGetter<string>,
  limit = 64,
) => {
  const cache = fetchCache<T>(category, limit);
  const fetch = useMemoize((url: string) => useFetch(url).json<T>(), { cache });

  let prevResult: UseFetchReturn<T> | undefined = undefined;
  const result = toRefs(
    toReactive(
      computedWithControl<UseFetchReturn<T>, string>(
        () => toValue(url),
        (prev) => {
          prevResult = prev;
          return fetch(toValue(url));
        },
      ),
    ),
  );

  return {
    ...result,
    data: computed(() => result.data.value ?? prevResult?.data.value),
  };
};
