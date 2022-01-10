import Vue from "vue";
import Vuex from "vuex";

Vue.use(Vuex);

export default new Vuex.Store({
  state: {
    sections: {
      error: false,
      tables: false,
    },
    listItem: "json-input",
  },
  mutations: {
    setSection(state, section) {
      this.state.sections[section.name] = section.value;
    },
    setListItem(state, listItem) {
      this.state.listItem = listItem;
    },
  },
  actions: {},
  modules: {},
});
