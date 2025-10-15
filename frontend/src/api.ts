import type { PublicConfig } from "@api/PublicConfig";
import type { Author, Collection, Platform, Tag } from "post-archiver";

export type Category = Platform | Tag | Collection | Author;

declare global {
  interface Window {
    PUBLIC_CONFIG: PublicConfig;
  }
}

export const usePublicConfig = () => window.PUBLIC_CONFIG;
export const loadPublicConfig = async () =>
  import.meta.env.DEV &&
  (window.PUBLIC_CONFIG ??= await fetch("/config.json").then((r) => r.json()));
