import Vue from "vue";
import Vuex from "vuex";

Vue.use(Vuex);

export default new Vuex.Store({
  state: {
    preview_data: undefined,
  },
  mutations: {
    set_preview_data(state, data) {
      state.preview_data = data;
    },
  },
  actions: {},
  modules: {},
});
