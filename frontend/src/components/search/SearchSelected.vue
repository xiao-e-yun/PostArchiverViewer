<script setup lang="ts">
import { remove } from "lodash";
import DoubleBadge from "../DoubleBadge.vue";
import { Tooltip, TooltipTrigger, TooltipContent } from "../ui/tooltip";
import { computed, inject, reactive } from "vue";
import {
  AuthorCategory,
  CollectionCategory,
  PlatformCategory,
  TagCategory,
  type Category,
} from "@/category";
import { SearchContextKey } from "./search";

const querys = inject(SearchContextKey)!;

const categories = computed(() => {
  const values: Category[] = reactive([]);

  for (const [builder, ids] of [
    [PlatformCategory, querys.value.platforms],
    [AuthorCategory, querys.value.authors],
    [TagCategory, querys.value.tags],
    [CollectionCategory, querys.value.collections],
  ] as const) {
    ids.forEach(async (id) => {
      const response = await builder.fromFetch(id);
      if (!response) return;
      values.push(response);
    });
  }

  return values;
});

const ORDER_BY = ["platforms", "authors", "tags", "collections"];
const sorted = computed(() =>
  categories.value
    .slice()
    .sort(
      (a, b) =>
        ORDER_BY.indexOf(a.builder.TYPE) - ORDER_BY.indexOf(b.builder.TYPE),
    ),
);
</script>

<template>
  <Tooltip
    v-for="category in sorted"
    :key="`${category.builder.TYPE}-${category.id}`"
  >
    <TooltipTrigger as-child>
      <DoubleBadge
        class="cursor-pointer text-xs h-6"
        @click="
          remove(querys[category.builder.TYPE], (item) => item === category.id)
        "
      >
        <template
          v-if="
            category.display()[1] && category.builder.TYPE !== 'collections'
          "
          #secondary
        >
          {{ category.display()[1] }}
        </template>
        {{ category.builder.PREFIX + category.display()[0] }}
      </DoubleBadge>
    </TooltipTrigger>
    <TooltipContent>
      Remove {{ category.builder.PREFIX + category.display()[0] }}
    </TooltipContent>
  </Tooltip>
</template>
