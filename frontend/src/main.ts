import { createApp } from 'vue'
import App from './App.vue'
import './assets/index.css'
import { createRouter, createWebHistory } from 'vue-router'

import Home from './pages/Home.vue'
import Author from './pages/Author.vue'
import Post from './pages/Post.vue'
import About from './pages/About.vue'
import Search from './pages/Search.vue'

const routes = [
  { path: '/', component: Home },
  { path: '/author/:id', component: Author },
  { path: '/post/:id', component: Post },
  { path: '/about', component: About },
  { path: '/search', component: Search },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})


createApp(App).use(router).mount('#app')
