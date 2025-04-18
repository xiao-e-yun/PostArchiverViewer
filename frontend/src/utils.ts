import { toValue, type MaybeRefOrGetter } from "vue";
import type { FileMeta } from "@api/FileMeta";
import { usePublicConfig } from "./api";

export function getFileMetaPath(fileMeta: FileMeta) {
  const config = usePublicConfig();
  const url = fileMeta.mime.startsWith("image/")
    ? (config.images_url ?? "/images")
    : (config.resource_url ?? "/resource");
  return (
    url.replace(/\/$/, "") +
    `/${fileMeta.author}/${fileMeta.post}/${fileMeta.filename}`
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
