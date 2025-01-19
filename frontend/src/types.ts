import type { Author, FileMeta, Post } from "post-archiver";

export type File = FileMeta & { url: string };
export type HasThumb = { thumb?: File };
export type AuthorsAPI = (Author & HasThumb)[];
export type PostsAPI = (Pick<Post, "id" | "author" | "title" | "updated"> & HasThumb)[];
export type PostAPI = Omit<Post, "content"> & HasThumb & { content: (string | File)[] };
export type InfoAPI = { authors: number, files: number, posts: number }