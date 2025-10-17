<script setup lang="ts">
import { reactive, ref } from "vue";
import { Card } from "../ui/card";
import { Button } from "../ui/button";
import { SearchIcon } from "lucide-vue-next";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "../ui/tooltip";
import { computed } from "vue";
import { last, remove } from "lodash";
import { computedAsync, refThrottled } from "@vueuse/core";
import { commitRef } from "@/utils";
import {
  Combobox,
  ComboboxAnchor,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "../ui/combobox";
import { Badge } from "../ui/badge";
import {
  AuthorCategory,
  categoryBuilders,
  CollectionCategory,
  PlatformCategory,
  TagCategory,
  type Category,
  type CategoryStatic,
} from "@/category";
import DoubleBadge from "../DoubleBadge.vue";

export interface SearchInput {
  search: string;
  collections: number[];
  platforms: number[];
  authors: number[];
  tags: number[];
}

const _model = defineModel<SearchInput>({ required: true });
const model = commitRef(_model);

const search = ref("");
const categories = computed(() => {
  const values: Category[] = reactive([]);

  for (const [builder, ids] of [
    [PlatformCategory, model.value.platforms],
    [AuthorCategory, model.value.authors],
    [TagCategory, model.value.tags],
    [CollectionCategory, model.value.collections],
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
const sortedCategories = computed(() =>
  categories.value
    .slice()
    .sort(
      (a, b) =>
        ORDER_BY.indexOf(a.builder.TYPE) - ORDER_BY.indexOf(b.builder.TYPE),
    ),
);

const prefixBuilder: Record<string, CategoryStatic> = Object.fromEntries(
  categoryBuilders.map((v) => [v.PREFIX, v]),
);

const keyword = refThrottled(
  computed(() => last(search.value.split(" ")) ?? ""),
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
  const keywords = search.value.split(" ");
  keywords.pop();
  search.value = keywords.join(" ") + (keywords.length ? " " : "");

  const list = model.value[category.builder.TYPE];
  if (list && !list.includes(category.id)) list.push(category.id);
};

const commit = model.commit;
const help = ref(false);
</script>

<template>
  <Card class="p-2">
    <TooltipProvider>
      <form class="flex gap-2" @submit.prevent="commit">
        <Combobox
          class="flex-1"
          :open="hasHints"
          ignore-filter
          :reset-search-term-on-blur="false"
          :reset-search-term-on-select="false"
        >
          <ComboboxAnchor as-child class="w-full">
            <ComboboxInput
              v-model="search"
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
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="outline" class="w-10 h-10">
              <SearchIcon />
            </Button>
          </TooltipTrigger>
          <TooltipContent> Search </TooltipContent>
        </Tooltip>
      </form>
      <div class="flex gap-2 mt-2 flex-wrap">
        <Tooltip
          v-for="category in sortedCategories"
          :key="`${category.builder.TYPE}-${category.id}`"
        >
          <TooltipTrigger as-child>
            <DoubleBadge
              class="cursor-pointer text-xs h-6"
              @click="
                remove(
                  model[category.builder.TYPE],
                  (item) => item === category.id,
                )
              "
            >
              <template
                v-if="
                  category.display()[1] &&
                  category.builder.TYPE !== 'collections'
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

        <Tooltip v-model:open="help">
          <TooltipTrigger as-child>
            <Button
              variant="secondary"
              class="ml-auto cursor-pointer text-xs rounded-full h-6 px-2.5 capitalize"
              @click="help = true"
            >
              ?
            </Button>
          </TooltipTrigger>
          <TooltipContent align="end">
            <template v-for="builder in categoryBuilders" :key="builder.TYPE">
              <code class="bg-secondary px-1 rounded-sm text-primary">{{
                builder.PREFIX
              }}</code>
              invokes {{ builder.TYPE }}.<br />
            </template>
            Click on labels to remove them.
          </TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  </Card>
</template>
