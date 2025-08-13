import type { InjectionKey, Ref } from "vue";

import type { PostResponse } from "@api/PostResponse";
import type { WithRelations } from "@api/WithRelations";
import type { useRelations } from "@/utils";
import type { FileMeta } from "post-archiver";
import type { ComputedRef } from "vue";

export const postKey = Symbol("post") as InjectionKey<{
  post: Ref<WithRelations<PostResponse> | null>;
  relations: ReturnType<typeof useRelations<PostResponse>>;
}>;

export const postImagesKey = Symbol("postImages") as InjectionKey<
  ComputedRef<FileMeta[]>
>;
