import { createRouter, createWebHistory } from 'vue-router'
import Home from "../views/Home.vue";
import GomokuAI from "../views/GomokuAI.vue";
import GomokuOnline from "../views/GomokuOnline.vue";

const routes = [
  {
    path: "/",
    name: "home",
    component: Home,
  },  {
    path: "/gomokuAI",
    name: "gomokuAI",
    component: GomokuAI,
  }, {
    path: "/gomokuOnline",
    name: "gomokuOnline",
    component: GomokuOnline,
  }
]

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

export default router
