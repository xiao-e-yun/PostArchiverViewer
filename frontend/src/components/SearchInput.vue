<script setup lang="ts">
import { reactive, ref, watch } from "vue";
import { Card } from "./ui/card";
import { Button } from "./ui/button";
import { SearchIcon } from "lucide-vue-next";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "./ui/tooltip";
import { computed } from "vue";
import { differenceBy, last } from "lodash";
import { asyncComputed, refThrottled, useMemoize } from "@vueuse/core";
import { fetch } from "ofetch";
import { getUrlWithParams } from "@/utils";
import type { WithRelations } from "@api/WithRelations";
import type { ListResponse } from "@api/ListResponse";
import type { Category } from "@/api";
import {
  Combobox,
  ComboboxAnchor,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "./ui/combobox";
import { Badge } from "./ui/badge";
import type { Author, Platform, Tag } from "post-archiver";

export interface SearchInputType {
  search: string;
  platforms: number[];
  authors: number[];
  tags: number[];
}

const emits = defineEmits<{
  (e: "update:modelValue", value: SearchInputType): void;
}>();
const props = defineProps<{ modelValue?: SearchInputType }>();

function update() {
  emits("update:modelValue", {
    search: search.value,
    platforms: categories.platforms.map((v) => v.id),
    authors: categories.authors.map((v) => v.id),
    tags: categories.tags.map((v) => v.id),
  });
}

const syncCategory = (
  category: keyof typeof categories,
  inputs?: SearchInputType,
) =>
  inputs &&
  inputs[category]
    .filter((id) => !categories[category].some((v) => v.id === id))
    .map(async (id) =>
      categories[category].push(await getCategory(category, id)),
    );
const getCategory = useMemoize(
  async (category: string, id: number) =>
    await (await fetch(`/api/${category}/${id}`)).json(),
);

const search = ref("");
const categories = reactive<{
  platforms: Category[];
  authors: Category[];
  tags: Category[];
}>({
  platforms: [],
  authors: [],
  tags: [],
});
watch(
  () => props.modelValue,
  (value) => {
    search.value = value?.search || "";
    categories.platforms = categories.platforms.filter(
      (v) => !value?.platforms || value.platforms.includes(v.id),
    );
    categories.authors = categories.authors.filter(
      (v) => !value?.authors || value.authors.includes(v.id),
    );
    categories.tags = categories.tags.filter(
      (v) => !value?.tags || value.tags.includes(v.id),
    );

    syncCategory("platforms", value);
    syncCategory("authors", value);
    syncCategory("tags", value);
  },
  { immediate: true },
);

const prefixCategories = {
  "&": "platforms",
  "@": "authors",
  "#": "tags",
} as const;

const categoryPrefixes = {
  platforms: "&",
  authors: "@",
  tags: "#",
} as const;

const hint = refThrottled(
  computed(() => {
    const keywordWithPrefix = last(search.value.split(" ")) ?? "";
    const prefix = keywordWithPrefix[0];
    const keyword = keywordWithPrefix.slice(1);

    const category = prefixCategories[prefix as keyof typeof prefixCategories];

    if (!category) return;
    return {
      category,
      prefix,
      keyword,
    };
  }),
  300,
);

const getHints = useMemoize(async (url: string, keyword?: string) => {
  const response = await fetch(getUrlWithParams(url, { search: keyword }));
  const data = (await response.json()) as WithRelations<ListResponse<Category>>;
  return data;
});

const categoryHints = asyncComputed(() => {
  const $hint = hint.value;
  if (!$hint) return null;

  const url = `/api/${$hint.category}`;
  return getHints(url, $hint.keyword);
});

const popKeywordFromSearch = () => {
  const keywords = search.value.split(" ");
  keywords.pop();
  search.value = keywords.join(" ") + (keywords.length > 0 ? " " : "");
};

const pushKeyword = (category: Author | Platform | Tag) => {
  popKeywordFromSearch();
  categories[hint.value!.category].push(category);
};

const categoriesList = computed(() =>
  (
    Object.entries(categories) as [
      "platforms" | "authors" | "tags",
      Category[],
    ][]
  )
    .map(([category, list]) => list.map((v) => [category, v] as const))
    .flat(),
);

const help = ref(false);
</script>

<template>
  <Card class="p-2">
    <TooltipProvider>
      <form class="flex gap-2" @submit.prevent="update">
        <Combobox
          class="flex-1"
          :open="!!hint"
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
          <ComboboxList v-if="!!hint" align="start" hide-when-detached>
            <template
              v-for="hints in differenceBy(
                categoryHints?.list,
                categories[hint.category] || [],
                'id',
              )"
              :key="hints.id"
            >
              <ComboboxItem
                :value="hints"
                class="capitalize flex justify-between"
                @select="pushKeyword(hints)"
              >
                {{ hint.prefix + hints.name }}
                <span
                  v-if="'platform' in hints && hints.platform !== null"
                  class="text-xs text-muted-foreground"
                >
                  {{
                    categories.platforms.find((h) => h.id === hints.platform)
                  }}
                </span>
              </ComboboxItem>
            </template>

            <ComboboxEmpty class="text-muted-foreground">
              <Badge variant="secondary">
                {{ hint.prefix + hint.keyword }}
              </Badge>
              no found.
            </ComboboxEmpty>
          </ComboboxList>
        </Combobox>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button variant="outline" class="w-10 h-10" @click="update">
              <SearchIcon />
            </Button>
          </TooltipTrigger>
          <TooltipContent> Search </TooltipContent>
        </Tooltip>
      </form>
      <div class="flex gap-2 mt-2 flex-wrap">
        <Tooltip
          v-for="[category, item] in categoriesList"
          :key="`${category}-${item.id}`"
        >
          <TooltipTrigger as-child>
            <Button
              variant="secondary"
              class="cursor-pointer text-xs rounded-full h-6 px-3 capitalize"
              @click="
                categories[category].splice(
                  categories[category].indexOf(item),
                  1,
                )
              "
            >
              {{ categoryPrefixes[category] + item.name }}
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            Remove {{ categoryPrefixes[category] + item.name }}
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
            <code class="bg-secondary px-1 rounded-sm text-primary">#</code>
            invokes tags<br />
            <code class="bg-secondary px-1 rounded-sm text-primary">@</code>
            invokes authors<br />
            <code class="bg-secondary px-1 rounded-sm text-primary">&#38;</code>
            invokes platforms<br />
            Click on labels to remove them.<br />
          </TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  </Card>
</template>
