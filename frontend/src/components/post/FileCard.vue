<script setup lang="ts">
import type { Component } from "vue";
import { RouterLink, useRouter } from "vue-router";
import { ArrowDown, FolderOpen } from "lucide-vue-next";

/**
 * A non-media file: icon, name, and actions on the right. Clicking the card
 * opens `to`, else downloads `href`, else emits `open`; `disabled` turns it off.
 * The `actions` slot replaces the default Download / Browse buttons.
 */
const props = defineProps<{
  name: string;
  icon: Component;
  href?: string;
  to?: string;
  /** Muted text left of the actions, e.g. the size. */
  detail?: string;
  disabled?: boolean;
}>();
const emit = defineEmits<{ open: [] }>();

const router = useRouter();
function open() {
  if (props.disabled) return;
  if (props.to) router.push(props.to);
  else if (props.href) window.open(props.href, "_blank", "noopener,noreferrer");
  else emit("open");
}
</script>

<template>
  <div
    class="w-full sm:w-md max-w-full flex items-center gap-3 p-4 transition-colors"
    :class="disabled ? 'cursor-default' : 'cursor-pointer hover:bg-accent/50'"
    @click="open"
  >
    <component :is="icon" class="w-12 h-12 shrink-0" />
    <div class="flex flex-col gap-2 min-w-0 flex-1">
      <span class="font-medium truncate" :title="name">{{ name }}</span>
      <div class="flex gap-2 items-center justify-end text-sm">
        <span
          v-if="detail"
          class="mr-auto text-muted-foreground tabular-nums truncate"
        >
          {{ detail }}
        </span>
        <slot name="actions">
          <a
            v-if="href"
            :href="href"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-1 rounded-md border bg-background px-2 sm:px-3 h-8 hover:bg-accent"
            title="Download"
            @click.stop
          >
            <ArrowDown class="w-4 h-4" />
            <!-- Icon-only on phones -->
            <span class="max-sm:sr-only">Download</span>
          </a>
          <RouterLink
            v-if="to"
            :to="to"
            class="inline-flex items-center gap-1 rounded-md px-2 sm:px-3 h-8 bg-primary text-primary-foreground hover:bg-primary/90"
            title="Browse"
            @click.stop
          >
            <FolderOpen class="w-4 h-4" />
            <span class="max-sm:sr-only">Browse</span>
          </RouterLink>
        </slot>
      </div>
    </div>
  </div>
</template>
