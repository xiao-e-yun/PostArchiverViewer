<script lang="ts" setup>
import { useFetch } from "@vueuse/core";
import { computed } from "vue";
import DoubleBadge from "./DoubleBadge.vue";
import type { WithRelations } from "@api/WithRelations";
import type { ListResponse } from "@api/ListResponse";
import type { Alias } from "post-archiver";
import { useRelations } from "@/utils";

const props = defineProps<{ id: number }>();

const url = computed(() => `/api/authors/${props.id}/aliases`);
const { data } = useFetch(url).json<WithRelations<ListResponse<Alias>>>();
const relations = useRelations(data);

const aliases = computed(() => data.value?.list || []);
const getPlatform = (alias: Alias) => {
  const platforms = relations.platforms;
  const platform = platforms.get(alias.platform)!;
  return { name: platform.name, link: `/platforms/${platform.id}` };
};
</script>

<template>
  <DoubleBadge
    v-for="(alias, index) in aliases"
    :key="index"
    :main="{ name: alias.source, link: alias.link }"
    :category="getPlatform(alias)"
  />
</template>
