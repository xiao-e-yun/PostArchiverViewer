import type { PostsAPI } from "@/api";
import {
  breakpointsTailwind,
  useBreakpoints,
  type EventBusIdentifier,
} from "@vueuse/core";
import { computed, type InjectionKey, type Ref } from "vue";

export const postsPrePageKey = "post-archiver-viewer.post-list.posts-per-page";

export const postListRestoreKey = Symbol(
  "postList:restore",
) as EventBusIdentifier<number>;

export interface PostListControl {
  total: Ref<PostsAPI["total"] | undefined>;
  postsPrePage: Ref<number | undefined>;
  pageIndex: Ref<number>;
}

export const postListControlKey = Symbol(
  "postList:control",
) as InjectionKey<PostListControl>;

export interface PostListContent {
  posts: Ref<PostsAPI["posts"] | undefined>;
  postsPrePage: Ref<number | undefined>;
}

export const postListContentKey = Symbol(
  "postList:content",
) as InjectionKey<PostListContent>;

export function usePostListControlUtils() {
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
