import type { commitRef } from "@/utils";
import { useRouteQuery } from "@vueuse/router";
import { forEach, mapValues } from "lodash";
import { computed, ref, toValue, type InjectionKey } from "vue";

export type SearchQuerys = {
  search: string;
  collections: number[];
  platforms: number[];
  authors: number[];
  tags: number[];
};

const asArray = <T>(v: T | T[]) => (Array.isArray(v) ? v : [v]);
const transform = <T>(v: T | T[]) => asArray(v).map(Number);

export const useSearchQuerys = (bindRouteQuery = true) => {
  if (!bindRouteQuery) {
    return ref<SearchQuerys>({
      search: "",
      collections: [],
      platforms: [],
      authors: [],
      tags: [],
    });
  }

  const rawQuerys = {
    search: useRouteQuery<string>("search", ""),
    collections: useRouteQuery("collections", [], { transform }),
    platforms: useRouteQuery("platforms", [], { transform }),
    authors: useRouteQuery("authors", [], { transform }),
    tags: useRouteQuery("tags", [], { transform }),
  };

  return computed<SearchQuerys>({
    set: (values) =>
      forEach(values, (v, k) => (rawQuerys[k as keyof SearchQuerys].value = v)),
    get: () => mapValues(rawQuerys, toValue) as unknown as SearchQuerys,
  });
};

export const SearchContextKey = Symbol("SearchContext") as InjectionKey<
  ReturnType<typeof commitRef<SearchQuerys>>
>;
