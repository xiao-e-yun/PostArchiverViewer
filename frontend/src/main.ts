import { createApp } from "vue";
import App from "./App.vue";
import "./assets/index.css";
import { createRouter, createWebHistory } from "vue-router";

import Home from "./pages/Home.vue";
import Author from "./pages/Author.vue";
import Post from "./pages/Post.vue";
import About from "./pages/About.vue";
import Search from "./pages/Search.vue";
import { setPublicConfig } from "./api";

(async () => {
  const routes = [
    { path: "/", component: Home },
    { path: "/author/:author", component: Author },
    { path: "/post/:post", component: Post },
    { path: "/about", component: About },
    { path: "/search", component: Search },
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

  await setPublicConfig();
  createApp(App).use(router).mount("#app");
})();
