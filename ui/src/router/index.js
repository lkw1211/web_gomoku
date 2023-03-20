import { createRouter, createWebHistory } from 'vue-router'
import Home from "../views/Home.vue";
import Gomoku from "../views/Gomoku.vue";

const routes = [
  {
    path: "/",
    name: "home",
    component: Home,
  },  {
    path: "/gomokuAI",
    name: "gomokuAI",
    component: Gomoku,
  },
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
