<script setup lang="ts" generic="U extends { id: number }">
import { computed, provide, toRef } from "vue";
import { Separator } from "../ui/separator";
import DynamicListControl from "./DynamicListControl.vue";
import { getUrlWithParams, useRelations, type UrlParams } from "@/utils";
import type { ListResponse } from "@api/ListResponse";
import {
  refThrottled,
  useEventListener,
  useFetch,
  useLocalStorage,
} from "@vueuse/core";

import { dynamicListControlKey, dynamicPrePageKey } from "./utils";
import { useRouteQuery } from "@vueuse/router";
import { cloneDeep, isEqual } from "lodash";
import type { WithRelations } from "@api/WithRelations";
import { watch } from "vue";

const props = withDefaults(
  defineProps<{
    url: string;
    query?: UrlParams;
    controls?: boolean;
    limit?: number;
  }>(),
  {
    query: () => ({}),
    limit: undefined,
    controls: true,
  },
);

const itemsPrePage = props.limit
  ? toRef(props, "limit")
  : useLocalStorage(dynamicPrePageKey, 20);
const pageIndex = useRouteQuery("page", "1", { transform: Number });
watch(
  [() => props.url, () => props.query, itemsPrePage],
  (value, old) => !isEqual(value, old) && (pageIndex.value = 1),
);

const pageThrottled = refThrottled(pageIndex, 500);

const url = computed(
  () =>
    getUrlWithParams(props.url, {
      page: pageThrottled.value - 1,
      limit: itemsPrePage.value,
      ...props.query,
    }).href,
);

const { data, isFetching: pending } = useFetch(url, {
  refetch: true,
}).json<WithRelations<ListResponse<U>>>();

const list = computed(() => data.value?.list);

const total = (() => {
  let prevProps = {};
  return computed<number>((prev) => {
    if (data.value) {
      prevProps = cloneDeep(props);
      return data.value.total;
    }

    if (isEqual(prevProps, props)) return prev as number;

    return 0;
  });
})();

provide(dynamicListControlKey, {
  itemsPrePage,
  pageIndex,
  pending,
  total,
});

const errorText = computed(() => {
  if (data.value) return "No found.";
  return "Something went wrong.";
});

const restorePageIndex = (value?: number) => {
  if (value) pageIndex.value = value;
};

useEventListener("popstate", () => restorePageIndex());
</script>

<template>
  <div v-if="pending || list?.length">
    <template v-if="controls">
      <DynamicListControl />
      <Separator class="my-4" />
    </template>
    <slot
      :list="list"
      :item-pre-page="itemsPrePage"
      :relations="useRelations(data)"
    />
    <template v-if="controls">
      <Separator class="my-4" />
      <DynamicListControl />
    </template>
  </div>
  <div v-else class="p-4 h-64">
    <h1 class="text-4xl font-bold my-4">{{ errorText }}</h1>
  </div>
</template>
