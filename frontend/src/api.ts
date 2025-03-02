import type { AuthorPostsJson } from "@api/AuthorPostsJson";
import type { AuthorJson } from "@api/AuthorJson";
import type { PostJson } from "@api/PostJson";
import type { InfoJson } from "@api/InfoJson";
import type { TagJson } from "@api/TagJson";

export type AuthorsAPI = AuthorJson[];
export type AuthorAPI = AuthorJson;
export type PostsAPI = AuthorPostsJson;
export type PostAPI = PostJson;
export type TagsAPI = TagJson[];
export type InfoAPI = InfoJson;
