/**
 * main.js
 *
 * Bootstraps Vuetify and other plugins then mounts the App`
 */

// Components
import App from './App.vue'

// Composables
import { createApp } from 'vue'

// Plugins
import { registerPlugins } from '@/plugins'

import { createStore } from 'vuex'

import { createRouter, createWebHistory } from 'vue-router'

import Home from "./views/Home.vue";
import About from "./views/About.vue";

const app = createApp(App)

registerPlugins(app)

const store = createStore({
  state () {
    return {
      sections: {
        error: false,
        tables: false,
      },
      listItem: "json-input",
    }
  },
  mutations: {
    setSection(state, section) {
      this.state.sections[section.name] = section.value;
    },
    setListItem(state, listItem) {
      this.state.listItem = listItem;
    },
  },
})

// Install the store instance as a plugin
app.use(store)


const routes = [
  {
    path: "/",
    name: "Home",
    component: Home,
  },
  {
    path: "/about",
    name: "About",
    component: About,
  },
];

const router = createRouter({
    // 4. Provide the history implementation to use. We are using the hash history for simplicity here.
    history: createWebHistory(),
    routes, // short for `routes: routes`
})

app.use(router)


app.mount('#app')
