export function getUrl(url: string | URL): URL {
  return new URL(url, location.origin);
}

export function getUrlWithParams(
  url: string | URL,
  params: Record<string, string | number | undefined>,
): URL {
  const urlObj = getUrl(url);
  Object.entries(params).forEach(([key, value]) => {
    if (value === undefined) return;
    urlObj.searchParams.set(key, String(value));
  });
  return urlObj;
}
