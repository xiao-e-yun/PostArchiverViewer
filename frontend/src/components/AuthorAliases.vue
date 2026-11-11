<script lang="ts" setup>
import { computed } from "vue";
import DoubleBadge from "./DoubleBadge.vue";
import type { Alias, Platform } from "post-archiver";

const props = defineProps<{
  aliases: Alias[];
  platforms: Map<number, Platform>;
}>();

const aliasesWithPlatforms = computed(() =>
  props.aliases.map((alias) => {
    const platform = props.platforms.get(alias.platform);
    return [
      alias,
      {
        name: platform?.name ?? "Unknown",
        link: platform && `/platforms/${platform.id}`,
      },
    ] as const;
  }),
);
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
