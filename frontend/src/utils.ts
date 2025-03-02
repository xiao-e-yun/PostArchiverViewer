import { toValue, type MaybeRefOrGetter } from "vue";

export function getUrl(url: string | URL): URL {
  return new URL(url, location.origin);
}

export function getUrlWithParams(
  url: string | URL,
  params: Record<string, MaybeRefOrGetter<string | number | undefined>>,
): URL {
  const urlObj = getUrl(url);
  Object.entries(params).forEach(([key, rawValue]) => {
    const value = toValue(rawValue);
    if (value === undefined) return;
    urlObj.searchParams.set(key, String(value));
  });
  return urlObj;
}

export function urlParamIntoString(param: string[] | string): string;
export function urlParamIntoString(param: undefined): undefined;
export function urlParamIntoString(param: string[] | string | undefined) {
  return Array.isArray(param) ? param[0] : param;
}
