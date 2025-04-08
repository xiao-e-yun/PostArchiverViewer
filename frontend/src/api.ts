import type { AuthorPostsJson } from "@api/AuthorPostsJson";
import type { AuthorJson } from "@api/AuthorJson";
import type { PostJson } from "@api/PostJson";
import type { InfoJson } from "@api/InfoJson";
import type { PublicConfig } from "@api/PublicConfig";

export type AuthorsAPI = AuthorJson[];
export type AuthorAPI = AuthorJson;
export type PostsAPI = AuthorPostsJson;
export type PostAPI = PostJson;
export type TagsAPI = Record<number, string>;
export type InfoAPI = InfoJson;

let publicConfig = {} as PublicConfig;
export const usePublicConfig = () => publicConfig;
export const setPublicConfig = async () =>
  (publicConfig = await fetch("/api/config.json").then((r) => r.json()));
