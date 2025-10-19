import { createApp, type Component } from "vue";
import App from "./App.vue";
import "./assets/index.css";
import { createRouter, createWebHistory } from "vue-router";

import Home from "./pages/Home.vue";
import Post from "./pages/Post.vue";
import About from "./pages/About.vue";
import Posts from "./pages/Posts.vue";
import { loadPublicConfig } from "./api";
import type { CategoryPostsContext } from "./pages/CategoryPosts.vue";
import Category from "./pages/Category.vue";
import CategoryPosts from "./pages/CategoryPosts.vue";
import Random from "./pages/Random.vue";

(async () => {
  const categoryRouter = (category: string, context: CategoryPostsContext) =>
    [
      { path: `/${category}s`, component: Category, props: context },
      { path: `/${category}s/:id`, component: CategoryPosts, props: context },
    ] as { path: string; component: Component; props: CategoryPostsContext }[];

  const routes = [
    { path: "/", component: Home },
    { path: "/posts", component: Posts },
    { path: "/posts/:id", component: Post },
    { path: "/random", component: Random },
    { path: "/about", component: About },
    ...categoryRouter("author", { category: "authors" }),
    ...categoryRouter("collection", { category: "collections" }),
    ...categoryRouter("platform", { category: "platforms" }),
    ...categoryRouter("tag", { category: "tags" }),
  ];

  const router = createRouter({
    history: createWebHistory(),
    routes,
    scrollBehavior(_, __, savedPosition) {
      if (savedPosition) {
        return new Promise((resolve) => {
          setTimeout(() => {
            resolve(savedPosition);
          }, 50);
        });
      } else {
        return { top: 0 };
      }
    },
  });

  await loadPublicConfig();
  createApp(App).use(router).mount("#app");
})();
