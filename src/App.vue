<template>
  <v-app>
    <v-navigation-drawer app width="350">
      <v-list-item to="/">
        <v-list-item-content>
          <img
            width="150"
            style="flex: none; margin-left: -10px"
            src="https://raw.githubusercontent.com/kindly/flatterer/main/docs/_static/flatterer-with-inline-text.svg"
          />
        </v-list-item-content>
      </v-list-item>

      <v-divider></v-divider>

      <v-list-item to="about">
        <v-list-item-content>
          <v-list-item-title class="subtitle-1"> About </v-list-item-title>
        </v-list-item-content>
      </v-list-item>

      <v-divider></v-divider>

      <v-list v-model="listItem" dense nav>
        <v-list-item-group v-model="listItem" v-if="$route.name == 'Home'">
          <v-list-item dense href="#json-input" value="json-input">
            JSON Input
          </v-list-item>
          <v-list-item dense href="#options" value="options">
            Options
          </v-list-item>
          <v-list-item v-if="sections.error" dense href="#error" value="error"
            >Error</v-list-item
          >
          <v-list-item
            v-if="sections.error"
            dense
            href="#input-data-preview"
            value="input-data-preview"
            >Input Data Preview</v-list-item
          >
          <v-list-item
            v-if="sections.tables"
            dense
            href="#tables-preview"
            value="tables-preview"
            >Tables</v-list-item
          >
          <v-list-item
            v-for="table in sections.tables"
            dense
            :key="table"
            :value="'table-' + table"
            :href="'#table-' + table"
            class="body-2 pl-10"
            style="min-height: 10px"
            >{{ table }}</v-list-item
          >
          <v-list-item
            v-if="sections.tables"
            dense
            href="#download"
            value="download"
            >Table Downloads</v-list-item
          >
        </v-list-item-group>
      </v-list>
    </v-navigation-drawer>

    <v-main>
      <router-view />
    </v-main>
  </v-app>
</template>

<script>
export default {
  name: "App",
  computed: {
    sections() {
      return this.$store.state.sections;
    },
    listItem: {
      get() {
        return this.$store.state.listItem;
      },
      set(value) {
        this.$store.commit("setListItem", value);
      },
    },
  },
  data: () => ({}),
};
</script>
