export function getUrl(url: string | URL): URL {
    return new URL(url,location.origin)
}

export function getUrlWithParams(url: string | URL, params: Record<string, any>): URL {
    const urlObj = getUrl(url)
    Object.entries(params).forEach(([key, value]) => {
        if (typeof value === undefined) return
        if (typeof value === 'object') value = JSON.stringify(value)
        urlObj.searchParams.set(key, value)
    })
    return urlObj
}