import type {
  Author,
  Collection,
  FileMeta,
  Platform,
  Tag,
} from "post-archiver";
import { getUrlWithParams } from "./utils";
export enum CategoryType {
  Tag = "tags",
  Author = "authors",
  Platform = "platforms",
  Collection = "collections",
}
export interface Category {
  id: number;
  name: string;
  updated?: string;
  source?: string | null;
  thumb?: FileMeta | null;
  platform?: Platform | null;

  display(): [string, string?];
  builder: CategoryStatic;
}

export interface CategoryStatic {
  // eslint-disable-next-line  @typescript-eslint/no-explicit-any
  new (value: any, relations: Relations): Category;
  TYPE: CategoryType;
  PREFIX: string;
  fromFetch(id: number): Promise<Category | null>;
  fromFetchList(
    search?: string,
    page?: number,
    limit?: number,
  ): Promise<Category[] | null>;
}

const fetchMap = new Map<string, Promise<Category | null>>();
const fromFetch = (
  builder: CategoryStatic,
): ((id: number) => Promise<Category | null>) => {
  return async (id: number): Promise<Category | null> => {
    const cacheKey = `${builder.TYPE}-${id}`;
    if (fetchMap.has(cacheKey)) {
      const cached = await fetchMap.get(cacheKey)!;
      if (cached) return cached;
    }

    const response: Promise<Category | null> = (async () => {
      const response = await fetch(`/api/${builder.TYPE}/${id}`);
      const data = await response.json();
      if (!data) return null;
      return new builder(data, data);
    })();

    fetchMap.set(cacheKey, response);
    return response;
  };
};

const fetchListMap = new Map<string, Promise<Category[] | null>>();
const fromFetchList = (
  builder: CategoryStatic,
): ((
  search?: string,
  page?: number,
  limit?: number,
) => Promise<Category[] | null>) => {
  return async (search: string = "", page: number = 0, limit: number = 20) => {
    const cacheKey = `${builder.TYPE}-${search}-${page}-${limit}`;
    if (fetchListMap.has(cacheKey)) {
      const cached = await fetchListMap.get(cacheKey)!;
      if (cached) return cached;
    }

    const response: Promise<Category[] | null> = (async () => {
      const url = getUrlWithParams(`/api/${builder.TYPE}`, {
        search,
        page,
        limit,
      });
      const response = await fetch(url);

      type ListResponse = WithRelations<{ list: Category[]; total: number }>;
      const data: ListResponse = await response.json();
      if (!data) return null;

      return data.list.map((item) => {
        const category = new builder(item, data);
        fetchMap.set(`${builder.TYPE}-${item.id}`, Promise.resolve(category));
        return category;
      });
    })();

    fetchListMap.set(cacheKey, response);
    return response;
  };
};

export class AuthorCategory implements Category {
  constructor(author: Author, relations: Relations) {
    this.id = author.id;
    this.name = author.name;
    this.updated = author.updated;
    this.thumb = getRelations(author.id, "file_metas", relations);
  }

  id: number;
  name: string;
  updated: string;
  thumb: FileMeta | null;

  display(): [string] {
    return [this.name];
  }

  builder = AuthorCategory;
  static TYPE = CategoryType.Author;
  static PREFIX = "@";
  static fromFetch = fromFetch(AuthorCategory);
  static fromFetchList = fromFetchList(AuthorCategory);
}

export class TagCategory implements Category {
  constructor(tag: Tag, relations: Relations) {
    this.id = tag.id;
    this.name = tag.name;
    this.platform = getRelations(tag.platform, "platforms", relations);
  }

  id: number;
  name: string;
  platform: Platform | null;

  display(): [string, string?] {
    return [this.name, this.platform?.name];
  }

  builder = TagCategory;
  static TYPE = CategoryType.Tag;
  static PREFIX = "#";
  static fromFetch = fromFetch(TagCategory);
  static fromFetchList = fromFetchList(TagCategory);
}

export class PlatformCategory implements Category {
  constructor(platform: Platform) {
    this.id = platform.id;
    this.name = platform.name;
  }

  id: number;
  name: string;

  display(): [string] {
    return [this.name];
  }

  builder = PlatformCategory;
  static TYPE = CategoryType.Platform;
  static PREFIX = ":";
  static fromFetch = fromFetch(PlatformCategory);
  static fromFetchList = fromFetchList(PlatformCategory);
}

export class CollectionCategory implements Category {
  constructor(collection: Collection, relations: Relations) {
    this.id = collection.id;
    this.name = collection.name;
    this.source = collection.source;
    this.thumb = getRelations(collection.thumb, "file_metas", relations);
  }

  id: number;
  name: string;
  thumb?: FileMeta | null;
  source?: string | null;

  display(): [string, string?] {
    return [this.name, this.source ?? undefined];
  }

  builder = CollectionCategory;
  static TYPE = CategoryType.Collection;
  static PREFIX = ".";
  static fromFetch = fromFetch(CollectionCategory);
  static fromFetchList = fromFetchList(CollectionCategory);
}

export const categoryBuilders: CategoryStatic[] = [
  PlatformCategory,
  AuthorCategory,
  TagCategory,
  CollectionCategory,
] as const;

export type RelationType =
  | "file_metas"
  | "platforms"
  | "tags"
  | "authors"
  | "collections";
export function getRelations(
  id: number | null,
  relation: "file_metas",
  relations: Relations,
): FileMeta | null;
export function getRelations(
  id: number | null,
  relation: "platforms",
  relations: Relations,
): Platform | null;
export function getRelations(
  id: number | null,
  relation: "tags",
  relations: Relations,
): Tag | null;
export function getRelations(
  id: number | null,
  relation: "authors",
  relations: Relations,
): Author | null;
export function getRelations(
  id: number | null,
  relation: "collections",
  relations: Relations,
): Collection | null;

export function getRelations(
  id: number | null,
  relation: RelationType,
  relations: Relations,
): FileMeta | Platform | Tag | Author | Collection | null {
  if (id === null) return null;
  return relations[relation]?.find((fm) => fm.id === id) ?? null;
}

export type WithRelations<T> = T & Relations;
export interface Relations {
  file_metas?: FileMeta[];
  platforms?: Platform[];
  tags?: Tag[];
  authors?: Author[];
  collections?: Collection[];
}
