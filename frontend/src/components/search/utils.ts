import type { UrlParams } from "@/utils";
import type { EventBusIdentifier } from "@vueuse/core";

export const searchRestoreKey = Symbol(
  "search:restore",
) as EventBusIdentifier<UrlParams>;
