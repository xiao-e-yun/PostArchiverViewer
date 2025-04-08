<script setup lang="ts">
import type { TagsAPI } from "@/api";
import {
  Combobox,
  ComboboxAnchor,
  ComboboxEmpty,
  ComboboxGroup,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox";
import {
  TagsInput,
  TagsInputInput,
  TagsInputItem,
  TagsInputItemDelete,
  TagsInputItemText,
} from "@/components/ui/tags-input";
import { useEventBus, useFetch } from "@vueuse/core";
import {
  useFilter,
  type AcceptableInputValue,
  type AcceptableValue,
} from "reka-ui";
import { computed, onUnmounted, ref, toValue } from "vue";
import { Button } from "../ui/button";
import { SearchIcon } from "lucide-vue-next";
import type { UrlParams } from "@/utils";
import { last } from "lodash";
import { searchRestoreKey } from "./utils";

const emit = defineEmits<{
  search: [UrlParams];
}>();

const { data: rawTags } = useFetch("/api/tags").json<TagsAPI>();
const tags = computed(
  () =>
    new Map(
      rawTags.value &&
        (Object.entries(rawTags.value).map((v) => v.reverse()) as [
          string,
          number,
        ][]),
    ),
);

const open = ref(false);
const search = ref<string>("");
const searchTags = ref<string[]>([]);

const { contains } = useFilter({ sensitivity: "base" });
const filteredTags = computed(() => {
  if (!tags.value) return [];
  const options = Array.from(tags.value.keys()).filter((i) =>
    searchTags.value.every((j) => j !== i),
  );

  const searchTag = last(search.value.split(" ")) ?? "";
  return options.filter((i) => contains(i, searchTag));
});

const wrapper = computed<AcceptableInputValue[]>({
  get: () => searchTags.value,
  set: (value) => (searchTags.value = value as string[]),
});

function update() {
  emit("search", {
    search: search.value,
    tags: searchTags.value.map((i) => tags.value.get(i)),
  });
}

const bus = useEventBus(searchRestoreKey);
const busStop = bus.on((querys) => {
  const tagNames = new Map(
    rawTags.value &&
      Object.entries(rawTags.value).map(([k, v]) => [parseInt(k), v]),
  );

  search.value = toValue((querys.search as string) ?? "");
  searchTags.value = toValue((querys.tags as number[]) ?? [])!
    .map((i) => tagNames.get(i))
    .filter((v) => v !== undefined);
});

onUnmounted(busStop);
</script>

<template>
  <Combobox
    v-model:open="open"
    v-model="wrapper as AcceptableValue"
    :reset-search-term-on-blur="false"
    :reset-search-term-on-select="false"
    :ignore-filter="true"
  >
    <ComboboxAnchor as-child>
      <TagsInput v-model="wrapper" class="px-2 gap-2 w-full">
        <div class="flex items-center gap-2 w-full">
          <ComboboxInput v-model="search" as-child @focus="open = true">
            <TagsInputInput
              placeholder="Search..."
              class="min-w-[200px] w-full py-0 px-1 border-none focus-visible:ring-0 h-auto"
              @keydown.enter.prevent
            />
          </ComboboxInput>
          <Button variant="outline" size="icon" type="button" @click="update">
            <SearchIcon />
          </Button>
        </div>

        <div class="flex gap-2 flex-wrap items-center w-full min-h-6">
          <TagsInputItem v-for="item in searchTags" :key="item" :value="item">
            <TagsInputItemText />
            <TagsInputItemDelete />
          </TagsInputItem>
        </div>
      </TagsInput>

      <ComboboxList align="start" class="w-60">
        <ComboboxEmpty />
        <ComboboxGroup>
          <ComboboxItem
            v-for="tag in filteredTags"
            :key="tag"
            :value="tag"
            @select.prevent="
              (ev) => {
                if (typeof ev.detail.value === 'string') {
                  const tag = ev.detail.value;
                  searchTags.push(tag);
                  search = search.replace(/[^ ]+$/, '');
                }

                if (filteredTags.length === 0) {
                  open = false;
                }
              }
            "
          >
            {{ tag }}
          </ComboboxItem>
        </ComboboxGroup>
      </ComboboxList>
    </ComboboxAnchor>
  </Combobox>
</template>
