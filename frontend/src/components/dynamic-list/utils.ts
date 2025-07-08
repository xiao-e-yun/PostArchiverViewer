import { breakpointsTailwind, useBreakpoints } from "@vueuse/core";
import { computed, type InjectionKey, type Ref } from "vue";

export const dynamicPrePageKey =
  "post-archiver-viewer.dynamic-list.posts-per-page";

export interface DynamicListControl {
  itemsPrePage: Ref<number | undefined>;
  pageIndex: Ref<number>;
  total: Ref<number>;
}

export const dynamicListControlKey = Symbol(
  "dynamicList:control",
) as InjectionKey<DynamicListControl>;

export function useDynamicListControlUtils() {
  const breakpoints = useBreakpoints(breakpointsTailwind);
  const smallMode = breakpoints.smallerOrEqual("sm");
  const siblingCount = computed(
    () =>
      ({
        sm: 1,
        md: 2,
        lg: 2,
        xl: 3,
        "2xl": 3,
        "": 0,
      })[breakpoints.active().value],
  );

  return {
    smallMode,
    siblingCount,
  };
}
