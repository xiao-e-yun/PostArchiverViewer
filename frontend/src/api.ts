import type { AuthorPostsJson } from "@api/AuthorPostsJson";
import type { AuthorJson } from "@api/AuthorJson";
import type { PostJson } from "@api/PostJson";
import type { SummaryJson } from "@api/SummaryJson";
import type { PublicConfig } from "@api/PublicConfig";

export type AuthorsAPI = AuthorJson[];
export type AuthorAPI = AuthorJson;
export type PostsAPI = AuthorPostsJson;
export type PostAPI = PostJson;
export type TagsAPI = Record<number, string>;
export type SummaryAPI = SummaryJson;

let publicConfig = {} as PublicConfig;
export const usePublicConfig = () => publicConfig;
export const setPublicConfig = async () =>
  (publicConfig = await fetch("/api/config.json").then((r) => r.json()));
