import type { PostAPI } from "@/api";
import type { InjectionKey, Ref } from "vue";

export interface PostOptions {
  post: Ref<PostAPI | null>;
}

export const postKey = Symbol("post") as InjectionKey<PostOptions>;
