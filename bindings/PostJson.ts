// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AuthorJson } from "./AuthorJson";
import type { Comment } from "./Comment";
import type { ContentJson } from "./ContentJson";
import type { FileMetaJson } from "./FileMetaJson";
import type { PostId } from "./PostId";
import type { TagJson } from "./TagJson";

export type PostJson = { id: PostId, author: AuthorJson, source: string | null, title: string, content: Array<ContentJson>, thumb: FileMetaJson | null, comments: Array<Comment>, updated: string, published: string, tags: Array<TagJson>, };