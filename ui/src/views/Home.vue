<template>
  <v-container>
    <v-card>
      <v-card-title>JSON Input</v-card-title>
      <v-container>
        <v-expansion-panels v-model="panel">
          <v-expansion-panel>
            <v-expansion-panel-header>Upload File</v-expansion-panel-header>

            <v-expansion-panel-content>
              <v-form ref="upload-form">
                <v-file-input
                  v-model="fileUpload"
                  label="File input"
                  id="file-input"
                  name="file"
                  outlined
                  dense
                  required
                ></v-file-input>
              </v-form>
            </v-expansion-panel-content>
          </v-expansion-panel>
          <v-expansion-panel>
            <v-expansion-panel-header
              >Download from URL</v-expansion-panel-header
            >
            <v-expansion-panel-content>
              <v-text-field
                outlined
                label="URL of JSON file"
                v-model="url"
                dense
                placeholder="https://link/to/file.json"
              ></v-text-field>
            </v-expansion-panel-content>
          </v-expansion-panel>
          <v-expansion-panel>
            <v-expansion-panel-header>Paste </v-expansion-panel-header>
            <v-expansion-panel-content>
              <v-textarea
                outlined
                v-model="paste"
                label="JSON data"
              ></v-textarea>
            </v-expansion-panel-content>
          </v-expansion-panel>
        </v-expansion-panels>
      </v-container>
      <v-container>
        <v-card>
          <v-container>
            <v-row>
              <v-col>
                <h3>Options</h3>
              </v-col>
            </v-row>
            <v-row class="mt-0">
              <v-col>
                <v-radio-group
                  class="mt-0"
                  v-model="arrayPosition"
                  row
                  mandatory
                  dense
                  messages="Position where main data array exists."
                >
                  <template v-slot:label>
                    <strong>Position in JSON:</strong>
                  </template>
                  <v-radio label="Top level array" value="top"></v-radio>
                  <v-radio label="JSON stream" value="stream"></v-radio>
                  <v-radio label="Array in object" value="nested"></v-radio>
                </v-radio-group>
              </v-col>
              <v-col>
                <v-text-field
                  outlined
                  dense
                  label="Key in object of data array"
                  v-model="array_key"
                  messages="The key in the object where the main array of objects exists."
                  :style="{
                    visibility:
                      arrayPosition == 'nested' ? 'visible' : 'hidden',
                  }"
                ></v-text-field>
              </v-col>
            </v-row>
            <v-row>
              <v-col>
                <v-text-field
                  outlined
                  dense
                  messages="Table name that represents main data array in input.  Defaults to `main`."
                  label="Main Table Name"
                  v-model="main_table_name"
                  placeholder="main"
                ></v-text-field>
              </v-col>
              <v-col>
                <v-checkbox
                  outlined
                  dense
                  hide-details="true"
                  v-model="inline_one_to_one"
                  label="Inline arrays with only single item"
                ></v-checkbox>
              </v-col>
            </v-row>
            <v-row>
              <v-col>
                <v-text-field
                  outlined
                  dense
                  label="JSONSchema URL"
                  v-model="json_schema"
                  placeholder="https://path/to/schema"
                  messages="URL where JSONSchema representing a single item in data array exists. If empty do not use schema"
                ></v-text-field>
              </v-col>
              <v-col>
                <v-select
                  :items="useTitle"
                  dense
                  label="Use titles from schema"
                  v-model="schemaTitle"
                  messages="Options for using titles within schema"
                  :style="{
                    visibility: json_schema.startsWith('http')
                      ? 'visible'
                      : 'hidden',
                  }"
                ></v-select>
              </v-col>
            </v-row>
            <v-row>
              <v-col>
                <v-text-field
                  outlined
                  dense
                  messages="Text prefixed to all output table names.  Defaults to no prefix."
                  label="Table Prefix"
                  v-model="table_prefix"
                ></v-text-field>
              </v-col>
              <v-col>
                <v-text-field
                  outlined
                  dense
                  label="Path Seperator"
                  v-model="path_seperator"
                  placeholder="_"
                  messages="Seperator between each part of the output field and table name. Defaults to `_`."
                ></v-text-field>
              </v-col>
            </v-row>
          </v-container>
        </v-card>
      </v-container>
      <v-container>
        <v-btn
          color="success"
          :disabled="submitButtonDisabled"
          @click="preview"
          >{{ submitButtonText }}</v-btn
        >
      </v-container>
    </v-card>
    <v-card class="mt-4" v-if="apiError">
      <v-alert prominent type="error">
        Server reported the following error:
        <br />
        <strong> {{ apiError }} </strong>
        <br />
        Try again with different options or data.
      </v-alert>
    </v-card>
    <v-card class="mt-4" v-if="fileStart">
      <v-card-title>Input data preview</v-card-title>
      <v-card-text
        >As the transformation failed, here is the initial part of the input
        file to check if it is as you expect:
        <v-sheet color="grey lighten-3 mt-1">
          <pre class="pa-2">{{ fileStart }}</pre>
        </v-sheet>
      </v-card-text>
    </v-card>
    <pre>
      {{ submitType }}
      {{ apiError }}
      {{ apiResponse }}
    </pre>
  </v-container>
</template>

<script>
export default {
  name: "Home",
  data: () => ({
    panel: 0,
    fileUpload: null,
    url: "",
    paste: "",
    useTitle: ["No Title", "Full Title", "Slug", "Underscore Slug"],
    arrayPosition: "top",
    array_key: "",
    path_seperator: "",
    table_prefix: "",
    schemaTitle: "No Title",
    json_schema: "",
    main_table_name: "",
    inline_one_to_one: false,

    fileStart: "",
    submitType: "",
    apiError: null,
    apiResponse: null,
  }),
  computed: {
    submitButtonText() {
      const lookup = {
        0: "Upload File and Preview",
        1: "Download URL and Preview",
        2: "Submit JSON and Preview",
      };
      return lookup[this.panel];
    },
    submitButtonDisabled() {
      const lookup = {
        0: this.fileUpload ? true : false,
        1: this.url.startsWith("http"),
        2: this.paste.length > 5,
      };
      return !lookup[this.panel];
    },
  },
  methods: {
    dataToParams() {
      let params = {};
      let schema_title = {
        "No Title": undefined,
        "Full Title": "full",
        Slug: "slug",
        "Underscore Slug": "underscore_slug",
      }[this.schemaTitle];
      if (schema_title) {
        params.schema_title = schema_title;
      }
      let simple_params = [
        "inline_one_to_one",
        "main_table_name",
        "table_prefix",
        "path_seperator",
        "array_key",
        "json_schema",
      ];
      for (var i in simple_params) {
        let key = simple_params[i];
        if (this[key]) {
          params[key] = this[key];
        }
      }
      return params;
    },
    preview() {
      let params = this.dataToParams();
      params.output_format = "preview";
      const lookup = {
        0: this.upload,
        1: this.downloadURL,
        2: this.submitPaste,
      };
      lookup[this.panel](params);
    },
    postToApi(urlParams, requestData) {
      requestData["method"] = "POST";
      requestData["headers"]["Accept"] = "application/json";
      this.apiStatus = null;
      this.apiError = null;
      this.apiResponse = null;
      this.fileStart = null;
      fetch("/api/convert?" + urlParams, requestData).then(
        function (response) {
          if (response.status != 200) {
            this.apiStatus = response.status;
            response.json().then(
              function (data) {
                this.apiError = data.error;
                this.id = data.id;
                this.fileStart = data.start;
              }.bind(this)
            );
          } else {
            response.json().then(
              function (data) {
                this.apiResponse = data;
                this.id = data.id;
              }.bind(this)
            );
          }
        }.bind(this)
      );
    },
    upload(params) {
      let urlParams = new URLSearchParams(params).toString();
      let formData = new FormData();
      formData.append("file", this.fileUpload);

      let requestData = {
        headers: {},
        body: formData,
      };
      this.postToApi(urlParams, requestData);
      this.submitType = "upload";
    },
    downloadURL(params) {
      params.file_url = this.url;
      let urlParams = new URLSearchParams(params).toString();
      let requestData = {
        method: "POST",
        headers: {},
      };
      this.postToApi(urlParams, requestData);
      this.submitType = "url";
    },
    submitPaste(params) {
      let urlParams = new URLSearchParams(params).toString();
      let requestData = {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: this.paste,
      };
      this.postToApi(urlParams, requestData);
      this.submitType = "paste";
    },
  },
};
</script>
