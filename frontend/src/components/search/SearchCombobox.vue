<script setup lang="ts">
import { computed, inject } from "vue";
import { SearchContextKey } from "./search";
import {
  Combobox,
  ComboboxAnchor,
  ComboboxInput,
  ComboboxEmpty,
  ComboboxList,
  ComboboxItem,
} from "../ui/combobox";
import {
  categoryBuilders,
  type Category,
  type CategoryStatic,
} from "@/category";
import { computedAsync, refThrottled } from "@vueuse/core";
import { last } from "lodash";
import { Badge } from "lucide-vue-next";

const querys = inject(SearchContextKey)!;

const prefixBuilder: Record<string, CategoryStatic> = Object.fromEntries(
  categoryBuilders.map((v) => [v.PREFIX, v]),
);

const keyword = refThrottled(
  computed(() => last(querys.value.search.split(" ")) ?? ""),
  300,
);
const hints = computedAsync(() => {
  const [prefix, $keyword] = [keyword.value[0], keyword.value.slice(1)];

  const builder = prefixBuilder[prefix];
  if (!builder) return null;

  return builder.fromFetchList($keyword);
}, null);
const hasHints = computed(() => hints.value !== null && hints.value.length > 0);

const pushKeyword = (category: Category) => {
  const keywords = querys.value.search.split(" ");
  keywords.pop();
  querys.value.search = keywords.join(" ") + (keywords.length ? " " : "");

  const list = querys.value[category.builder.TYPE];
  if (list && !list.includes(category.id)) list.push(category.id);
};
</script>

<template>
  <Combobox
    class="flex-1"
    :open="hasHints"
    ignore-filter
    :reset-search-term-on-blur="false"
    :reset-search-term-on-select="false"
  >
    <ComboboxAnchor as-child class="w-full">
      <ComboboxInput
        v-model="querys.search"
        placeholder="Search..."
        class="h-full"
      />
    </ComboboxAnchor>
    <ComboboxList align="start" class="w-[100%]" hide-when-detached>
      <ComboboxItem
        v-for="hint in hints"
        :key="hint.id"
        :value="hints"
        class="capitalize flex justify-between"
        :style="{ 'padding-bottom': hint.display()[1] && '1.2rem' }"
        @select="pushKeyword(hint)"
      >
        {{ hint.builder.PREFIX + hint.display()[0] }}
        <span
          class="text-xs text-muted-foreground absolute right-2 bottom-1 max-w-[80%] overflow-hidden text-ellipsis whitespace-nowrap"
        >
          {{ hint.display()[1] }}
        </span>
      </ComboboxItem>

      <ComboboxEmpty class="text-muted-foreground">
        {{ hints ? "No results found." : "Loading..." }}
        <Badge variant="secondary"> {{ keyword }} </Badge>
      </ComboboxEmpty>
    </ComboboxList>
  </Combobox>
</template>
