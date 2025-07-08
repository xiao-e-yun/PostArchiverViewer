import type { InjectionKey, Ref } from "vue";

import type { PostResponse } from "@api/PostResponse";
import type { WithRelations } from "@api/WithRelations";
import type { useRelations } from "@/utils";

export const postKey = Symbol("post") as InjectionKey<{
  post: Ref<WithRelations<PostResponse> | null>;
  relations: ReturnType<typeof useRelations<PostResponse>>;
}>;
