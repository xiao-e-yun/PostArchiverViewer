<script lang="ts" setup>
import { Badge } from "@/components/ui/badge";
import { RouterLink } from "vue-router";

defineProps<{
  category: {
    name: string;
    link?: string | null;
  };
  main: {
    name: string;
    link?: string | null;
  };
}>();
</script>

<template>
  <component :is="main.link ? 'a' : 'div'" :href="main.link">
    <Badge class="relative py-0 pl-0 gap-1 border-none" variant="secondary">
      <component :is="category.link ? RouterLink : 'div'" :to="category.link">
        <Badge class="capitalize">
          {{ category.name }}
        </Badge>
      </component>
      {{ main.name }}
    </Badge>
  </component>
</template>
