import type { PublicConfig } from "@api/PublicConfig";
import type { Author, Collection, Platform, Tag } from "post-archiver";

export type Category = Platform | Tag | Collection | Author;

let publicConfig = {} as PublicConfig;
export const usePublicConfig = () => publicConfig;
export const setPublicConfig = async () =>
  (publicConfig = await fetch("/api/config.json").then((r) => r.json()));
