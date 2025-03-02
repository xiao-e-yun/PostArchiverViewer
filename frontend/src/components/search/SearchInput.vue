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
import type { TagJson } from "@api/TagJson";
import { useFetch } from "@vueuse/core";
import {
  useFilter,
  type AcceptableInputValue,
  type AcceptableValue,
} from "reka-ui";
import { computed, ref } from "vue";
import { Button } from "../ui/button";
import { SearchIcon } from "lucide-vue-next";
import type { UrlParams } from "@/utils";

const emit = defineEmits<{
  search: [UrlParams];
}>();

const { data: rawTags } = useFetch("/api/tags").json<TagsAPI>();
const tags = computed(() => new Map(rawTags.value));

const open = ref(false);
const search = ref<string>("");
const searchTags = ref<TagJson[]>([]);

const { contains } = useFilter({ sensitivity: "base" });
const filteredTags = computed(() => {
  const tags = rawTags.value;
  if (!tags) return [];
  const options = tags?.filter((i) =>
    searchTags.value.every((j) => j[0] !== i[0]),
  );
  return search.value
    ? options.filter((option) => contains(option[1], search.value.trim()))
    : options;
});

const wrapper = computed<AcceptableInputValue[]>({
  get: () => searchTags.value.map((i) => i[0].toString()),
  set: (value) =>
    (searchTags.value = value.map((i) => {
      const id = parseInt(i as string);
      return [id, tags.value.get(id)!];
    })),
});

function update() {
  emit("search", {
    search: search.value,
    tags: searchTags.value.map((i) => i[0]),
  });
}
</script>

<template>
  <Combobox
    v-model:open="open"
    v-model="wrapper as AcceptableValue"
    :reset-search-term-on-blur="false"
    :ignore-filter="true"
  >
    <ComboboxAnchor as-child>
      <TagsInput
        v-model="wrapper"
        class="px-2 gap-2 w-full"
        :display-value="(i) => tags.get(parseInt(i as string))!"
      >
        <div class="flex items-center gap-2 w-full">
          <ComboboxInput v-model="search" as-child>
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
          <TagsInputItem
            v-for="item in searchTags"
            :key="item[0]"
            :value="item[0].toString()"
          >
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
            :key="tag[0]"
            :value="tag[0].toString()"
            @select.prevent="
              (ev) => {
                if (typeof ev.detail.value === 'string') {
                  const id = parseInt(ev.detail.value);
                  searchTags.push([id, tags.get(id)!]);
                  search = '';
                }

                if (filteredTags.length === 0) {
                  open = false;
                }
              }
            "
          >
            {{ tag[1] }}
          </ComboboxItem>
        </ComboboxGroup>
      </ComboboxList>
    </ComboboxAnchor>
  </Combobox>
</template>
