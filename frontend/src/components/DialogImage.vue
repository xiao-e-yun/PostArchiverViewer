<script setup lang="ts">
import { cn } from "@/lib/utils";
import { Minus, Plus, Search, X } from "lucide-vue-next";
import {
  DialogClose,
  DialogContent,
  type DialogContentEmits,
  type DialogContentProps,
  DialogOverlay,
  DialogPortal,
  useForwardPropsEmits,
} from "radix-vue";
import {
  computed,
  nextTick,
  ref,
  useTemplateRef,
  type HTMLAttributes,
} from "vue";
import { Dialog, DialogTitle } from "./ui/dialog";
import { useThrottleFn, useToggle } from "@vueuse/core";

const props = defineProps<
  DialogContentProps & {
    class?: HTMLAttributes["class"];
    src: string;
    aspect?: number;
  }
>();
const emits = defineEmits<DialogContentEmits>();

const delegatedProps = computed(() => {
  const { class: _, ...delegated } = props;

  return delegated;
});

const forwarded = useForwardPropsEmits(delegatedProps, emits);

const overlay = useTemplateRef<typeof DialogOverlay>("overlay");
const dialog = useTemplateRef<typeof DialogContent>("dialog");

const MIN_SCALE = 1;
const MAX_SCALE = 5.0;

const scale = ref(MIN_SCALE);
const limitScale = (value: number) =>
  Math.max(MIN_SCALE, Math.min(MAX_SCALE, value));

const setScale = (value: number) => scale.value = limitScale(scale.value + value);
const whellSize = useThrottleFn((e: WheelEvent) => {
  const prevScale = scale.value;
  setScale(-e.deltaY / 100 / 2)

  const $overlay = overlay.value;
  const $dialog = dialog.value;
  if (!$overlay || !$dialog) return;
  const overlayEl = $overlay.$el as HTMLDivElement;
  const dialogEl = $dialog.$el as HTMLDivElement;

  const scaleDiff = 1 + scale.value - prevScale;
  if (scaleDiff === 1) return;

  if (e.deltaY < 0) {
    nextTick(() => {
      const x = e.x - overlayEl.clientWidth / 2;
      const y = e.y - overlayEl.clientHeight / 2;

      overlayEl.scrollBy({
        left: x * scaleDiff,
        top: y * scaleDiff,
      });
    });
  } else {
    nextTick(() => {
      const scrollX = overlayEl.scrollLeft * scaleDiff;
      const scrollY = overlayEl.scrollTop * scaleDiff;

      const centerX =
        (dialogEl.clientWidth - overlayEl.clientWidth) / 2 +
        dialogEl.offsetLeft;
      const centerY =
        (dialogEl.clientHeight - overlayEl.clientHeight) / 2 +
        dialogEl.offsetTop;

      const x = (centerX + scrollX) / 2;
      const y = (centerY + scrollY) / 2;

      overlayEl.scrollTo({
        left: x,
        top: y,
      });
    });
  }
}, 20);

const style = computed(() => ({
  height: `${scale.value * 80}vh`,
  aspectRatio: props.aspect,
}));

const displayScale = computed(() => (scale.value * 100).toFixed());
const [tooltipScale, toggleTooltipScale] = useToggle(true)
</script>

<template>
  <Dialog>
    <slot />
    <DialogPortal>
      <DialogOverlay
        class="fixed inset-0 z-50 grid place-items-center overflow-y-auto bg-black/80 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
        @wheel.ctrl.prevent="whellSize"
        ref="overlay"
      >
        <DialogContent
          :style="style"
          :class="
            cn(
              'relative z-50 grid w-auto max-w-none my-8 gap-4 border border-border bg-background p-6 shadow-lg sm:rounded-lg',
              props.class
            )
          "
          v-bind="forwarded"
          @pointer-down-outside="(event) => {
                const originalEvent = event.detail.originalEvent;
                const target = originalEvent.target as HTMLElement;
                if (originalEvent.offsetX > target.clientWidth || originalEvent.offsetY > target.clientHeight) {
                    event.preventDefault();
                }
            }"
          ref="dialog"
        >
          <img :src="src" :style="style" />
          <DialogTitle class="sr-only">{{ src }} of image</DialogTitle>
          <DialogClose
            class="absolute top-3 right-3 p-0.5 transition-colors rounded-md hover:bg-secondary"
          >
            <X class="w-6 h-6" />
            <span class="sr-only">Close</span>
          </DialogClose>

          <div
            class="fixed bottom-5 left-3 z-[100] bg-background p-1 text-base rounded-md flex border"
          >
            <div class="flex gap-1 items-center">
              <Search class="w-6 h-6 p-0.5 hover:bg-secondary rounded-sm" @click="toggleTooltipScale()" />
              <template v-if="tooltipScale">
                <span class="align-baseline">{{ displayScale }} %</span>
                <Plus class="w-6 h-6 p-0.5 hover:bg-secondary rounded-sm" @click="setScale(0.5)" />
                <Minus class="w-6 h-6 p-0.5 hover:bg-secondary rounded-sm" @click="setScale(-0.5)" />
              </template>
            </div>
          </div>
        </DialogContent>
      </DialogOverlay>
    </DialogPortal>
  </Dialog>
</template>
