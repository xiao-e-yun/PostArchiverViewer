import { useRouteQuery } from "@vueuse/router";
import { forEach, mapValues } from "lodash";
import { computed, toValue } from "vue";

const asArray = <T>(v: T | T[]) => (Array.isArray(v) ? v : [v]);
const transform = <T>(v: T | T[]) => asArray(v).map(Number);

type Querys = {
  search: string;
  collections: number[];
  platforms: number[];
  authors: number[];
  tags: number[];
};

export const useSearchQuerys = () => {
  const rawQuerys = {
    search: useRouteQuery<string>("search", ""),
    collections: useRouteQuery("collections", [], { transform }),
    platforms: useRouteQuery("platforms", [], { transform }),
    authors: useRouteQuery("authors", [], { transform }),
    tags: useRouteQuery("tags", [], { transform }),
  };

  return computed<Querys>({
    set: (values) =>
      forEach(values, (v, k) => (rawQuerys[k as keyof Querys].value = v)),
    get: () => mapValues(rawQuerys, toValue) as unknown as Querys,
  });
};
