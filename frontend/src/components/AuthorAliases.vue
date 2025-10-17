<script lang="ts" setup>
import { computed } from "vue";
import DoubleBadge from "./DoubleBadge.vue";
import type { WithRelations } from "@api/WithRelations";
import type { ListResponse } from "@api/ListResponse";
import type { Alias } from "post-archiver";
import { useFetchWithCache, useRelations } from "@/utils";

const props = defineProps<{ id: number }>();

const url = computed(() => `/api/authors/${props.id}/aliases`);
const { data } = useFetchWithCache<WithRelations<ListResponse<Alias>>>(
  "aliases",
  url,
);
const relations = useRelations(data);

const aliases = computed<Alias[]>(() => data.value?.list || []);
const aliasesWithPlatforms = computed(() =>
  aliases.value.map((a) => [a, getPlatform(a)] as const),
);

const getPlatform = (alias: Alias) => {
  const platforms = relations.platforms;
  const platform = platforms.get(alias.platform)!;
  return { name: platform.name, link: `/platforms/${platform.id}` };
};
</script>

<template>
  <DoubleBadge
    v-for="([alias, platform], index) in aliasesWithPlatforms"
    :key="index"
    :link="alias.link ?? undefined"
    :secondary-link="platform.link"
  >
    {{ alias.source }}
    <template #secondary>{{ platform.name }}</template>
  </DoubleBadge>
</template>
