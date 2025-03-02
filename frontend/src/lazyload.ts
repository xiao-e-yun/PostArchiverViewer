import { createGlobalState } from "@vueuse/core";
import LazyLoad from "vanilla-lazyload";

export const useLazyLoad = createGlobalState(() => new LazyLoad());
