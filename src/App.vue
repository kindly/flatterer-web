<template>
  <v-app>
    <v-navigation-drawer app width="350">
      <v-list-item to="/">
        <img
          width="150"
          style="flex: none"
          src="https://raw.githubusercontent.com/kindly/flatterer/main/docs/_static/flatterer-with-inline-text.svg"
        />
      </v-list-item>

      <v-divider></v-divider>

      <v-list-item to="about">
        <v-list-item-title class="subtitle-1"> About </v-list-item-title>
      </v-list-item>

      <v-divider></v-divider>

      <v-list dense nav>
        <v-list-item-group v-if="$route.name == 'Home'" v-model="listItem">
          <v-list-item dense href="#json-input" :active="listItem == 'json-input'">
            JSON Input
          </v-list-item>
          <v-list-item dense href="#options" :active="listItem == 'options'">
            Options
          </v-list-item>
          <v-list-item v-if="sections.error" dense href="#error" :active="listItem == 'error'"
            >Error</v-list-item
          >
          <v-list-item
            v-if="sections.error"
            density="compact"
            href="#input-data-preview"
            :active="listItem == 'input-data-preview'"
            >Input Data Preview</v-list-item
          >
          <v-list-item
            v-if="sections.tables"
            density="compact"
            href="#tables-preview"
            :active="listItem == 'tables-preview'"

            >Tables</v-list-item
          >
          <v-list-item
            v-for="table in sections.tables"
            density="compact"
            :key="table"
            :active="listItem == 'table-' + table"
            :href="'#table-' + table"
            class="body-2 pl-10"
            style="min-height: 10px"
            >{{ table }}</v-list-item
          >
          <v-list-item
            v-if="sections.tables"
            density="compact"
            href="#download"
            value="download"
            :active="listItem == 'download'"
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