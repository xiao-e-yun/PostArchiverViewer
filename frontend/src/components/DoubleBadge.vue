<script lang="ts" setup>
import { Badge } from "@/components/ui/badge";
import { RouterLink } from "vue-router";

defineProps<{
  link?: string;
  secondaryLink?: string;
}>();

const slots = defineSlots<{
  default: () => unknown;
  secondary?: () => unknown;
}>();
</script>

<template>
  <Badge
    variant="secondary"
    class="relative py-0 border-none capitalize flex"
    :style="{ 'padding-left': slots.secondary && '0' }"
    as-child
  >
    <component :is="link ? 'a' : 'div'" :href="link">
      <Badge v-if="slots.secondary" as-child class="mr-1 text-[length:inherit]">
        <component
          :is="secondaryLink ? RouterLink : 'div'"
          :to="secondaryLink"
          class="capitalize"
        >
          <slot name="secondary" />
        </component>
      </Badge>
      <slot />
    </component>
  </Badge>
</template>
