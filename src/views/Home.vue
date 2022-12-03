<template>
  <v-container>
    <v-card>
      <v-row>
        <v-col>
          <v-card-title id="json-input" v-intersect="onIntersect"
            >JSON Input</v-card-title
          >
        </v-col>
        <v-col>
          <v-container>
            <v-btn
              class="float-right"
              color="deep-orange lighten-4"
              :disabled="formState === 'new'"
              @click="reset"
              >reset all</v-btn
            >
          </v-container>
        </v-col>
      </v-row>
      <v-container>
        <v-expansion-panels v-model="panel">
          <v-expansion-panel>
            <v-expansion-panel-title>Upload File</v-expansion-panel-title>

            <v-expansion-panel-text class="pt-6">
              <v-file-input
                v-on:change="uploadFile($event, 'fileUpload')"
                v-on:click:clear="uploadFile($event, 'fileUpload')"
                label="File input"
                id="file-input"
                name="file"
                outlined
                dense
                required
              ></v-file-input>
            </v-expansion-panel-text>
          </v-expansion-panel>
          <v-expansion-panel>
            <v-expansion-panel-title
              >Download from URL</v-expansion-panel-title
            >
            <v-expansion-panel-text class="pt-6">
              <v-text-field
                outlined
                label="URL of JSON file"
                v-model="url"
                dense
                placeholder="https://link/to/file.json"
              ></v-text-field>
            </v-expansion-panel-text>
          </v-expansion-panel>
          <v-expansion-panel>
            <v-expansion-panel-title>Paste</v-expansion-panel-title>
            <v-expansion-panel-text class="pt-6">
              <v-textarea
                outlined
                v-model="paste"
                label="JSON data"
              ></v-textarea>
            </v-expansion-panel-text>
          </v-expansion-panel>
        </v-expansion-panels>
      </v-container>
      <v-container>
        <v-card>
          <v-container>
            <v-row>
              <v-col>
                <h3 id="options" v-intersect="onIntersect">Options</h3>
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
                  <v-radio label="Guess based on data" value="top"></v-radio>
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
                  :disabled="!json_schema.startsWith('http')"
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
              <v-col>
                <v-text-field
                  outlined
                  dense
                  label="Pushdown"
                  v-model="pushdown"
                  placeholder="id"
                  messages="Field to pushdown to seperate tables"
                ></v-text-field>
              </v-col>
            </v-row>
            <v-row v-if="panel != 2">
              <v-col>
                <v-file-input
                  v-on:change="uploadFile($event, 'fieldsUpload')"
                  v-on:click:clear="uploadFile($event, 'fieldsUpload')"
                  label="fields.csv file"
                  id="fields-file"
                  name="fields"
                  outlined
                  dense
                  required
                ></v-file-input>
              </v-col>
              <v-col>
                <v-checkbox
                  outlined
                  dense
                  hide-details="true"
                  v-model="fields_only"
                  label="Only output fields in file"
                ></v-checkbox>
              </v-col>
              <v-col>
                <v-file-input
                  v-on:change="uploadFile($event, 'tablesUpload')"
                  v-on:click:clear="uploadFile($event, 'tablesUpload')"
                  label="tables.csv file"
                  id="tables-file"
                  name="tables"
                  outlined
                  dense
                  required
                ></v-file-input>
              </v-col>
              <v-col>
                <v-checkbox
                  outlined
                  dense
                  hide-details="true"
                  v-model="tables_only"
                  label="Only output tables in file"
                ></v-checkbox>
              </v-col>
            </v-row>
          </v-container>
        </v-card>
      </v-container>
      <v-container>
        <v-btn
          color="success"
          :disabled="submitButtonDisabled || formState == 'submitted'"
          @click="preview"
          >{{ submitButtonText }}</v-btn
        >
        <v-progress-circular
          indeterminate
          color="grey"
          class="ml-4"
          v-if="!apiStatus && formState == 'submitted'"
        ></v-progress-circular>
      </v-container>
    </v-card>
    <v-card id="error" v-intersect="onIntersect" class="mt-4" v-if="apiError">
      <v-alert prominent type="error">
        Server reported the following error:
        <br />
        <strong> {{ apiError }} </strong>
        <br />
        Try again with different options or data.
      </v-alert>
    </v-card>
    <v-card class="mt-4" v-if="fileStart">
      <v-card-title id="input-data-preview" v-intersect="onIntersect"
        >Input data preview</v-card-title
      >
      <v-card-text
        >As the transformation failed, here is the initial part of the input
        file to check if it is as you expect:
        <v-sheet color="grey lighten-3 mt-1">
          <pre class="pa-2" style="white-space: pre-wrap">{{ fileStart }}</pre>
        </v-sheet>
      </v-card-text>
    </v-card>

    <v-card class="mt-4" v-if="apiResponse" id="success">
      <v-alert type="success"
        >File Processed Successfully!
        <small v-if="apiResponse.guess_text"
          >Guessed that data array was in {{ apiResponse.guess_text }}
        </small>
      </v-alert>
    </v-card>

    <v-card class="mt-4" v-if="apiResponse">
      <v-card-title id="tables-preview" v-intersect="onIntersect"
        >Tables Preview</v-card-title
      >
      <v-card-text>
        Below is a preview of the tables that will be created.
      </v-card-text>
      <v-container v-for="table in apiResponse.preview" :key="table.table_name">
        <v-card :id="'table-' + table.table_name" v-intersect="onIntersect">
          <v-card-title class="subtitle-1">
            {{ table.table_name }}
          </v-card-title>

          <v-table density="compact">
            <thead>
              <tr>
                <th v-for="heading in fieldHeaders" :key="heading.value" class="text-left">
                  {{heading.text}}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="row in table.fields"
                :key="row.field_title"
              >
                <td v-for="heading in fieldHeaders" :key="heading.value">{{ row[heading.value] }}</td>
              </tr>
            </tbody>
          </v-table>
        </v-card>
      </v-container>
    </v-card>

    <v-card class="mt-4" v-if="apiResponse" id="download">
      <v-container>
        <v-row>
          <v-col>
            <v-btn color="success" :href="generateDownload('zip')"
              >Download Full Zip</v-btn
            >
          </v-col>
          <v-col>
            <v-btn color="success" :href="generateDownload('xlsx')"
              >Download XLSX</v-btn
            >
          </v-col>
          <v-col>
            <v-btn color="success" :href="generateDownload('sqlite')"
              >Download SQLite</v-btn
            >
          </v-col>
          <v-col>
            <v-btn color="success" :href="generateDownload('csv')"
              >{{ main_table_name || "main" }} table as CSV</v-btn
            >
          </v-col>
          <v-col>
            <v-btn color="success" :href="generateDownload('fields')"
              >Download fields.csv</v-btn
            >
          </v-col>
          <v-col>
            <v-btn color="success" :href="generateDownload('tables')"
              >Download tables.csv</v-btn
            >
          </v-col>
        </v-row>
      </v-container>
    </v-card>
  </v-container>
</template>

<script>
function defaultData() {
  return {
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
    fieldsUpload: null,
    fields_only: false,
    tablesUpload: null,
    tables_only: false,
    pushdown:"",
    id: "",
    formState: "new",
    fileStart: "",
    submitType: "",
    apiError: "",
    apiResponse: null,
    apiStatus: null,

    fieldHeaders: [
      { text: "Field Name", value: "field_title" },
      { text: "Field Type", value: "field_type" },
      { text: "Row Count", value: "count" },
      { text: "Value in first row", value: "row 0" },
      { text: "Value in second row", value: "row 1" },
      { text: "Value in third row", value: "row 2" },
    ],
  };
}

export default {
  name: "Home",
  data: defaultData,
  watch: {
    apiError(newError) {
      this.$store.commit("setSection", {
        name: "error",
        value: newError ? true : false,
      });
    },
    apiResponse(newResponse) {
      let value = false;
      if (newResponse) {
        value = newResponse.preview.map((value) => {
          return value.table_name;
        });
      }
      this.$store.commit("setSection", { name: "tables", value });
    },
    formChanged() {
      this.formState = "changed";
      this.id = "";
      this.fileStart = "";
      this.submitType = "";
      this.apiError = "";
      this.apiResponse = null;
      this.apiStatus = null;
    },
  },
  computed: {
    formChanged() {
      return [
        this.panel,
        this.fileUpload,
        this.url,
        this.paste,
        this.arrayPosition,
        this.array_key,
        this.path_seperator,
        this.table_prefix,
        this.schemaTitle,
        this.json_schema,
        this.main_table_name,
        this.inline_one_to_one,
        this.fieldsUpload,
        this.fields_only,
        this.tablesUpload,
        this.tables_only,
        this.pushdown,
      ];
    },
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
    reset() {
      const data = defaultData();
      Object.keys(data).forEach((k) => (this[k] = data[k]));
      this.$nextTick(() => {
        this.formState = "new";
      });
    },
    dataToParams() {
      let params = {};
      let schema_title = {
        "No Title": undefined,
        "Full Title": "full",
        Slug: "slug",
        "Underscore Slug": "underscore_slug",
      }[this.schemaTitle];
      if (schema_title) {
        params.schema_titles = schema_title;
      }
      let simple_params = [
        "inline_one_to_one",
        "main_table_name",
        "table_prefix",
        "path_seperator",
        "array_key",
        "json_schema",
        "fields_only",
        "tables_only",
        "pushdown",
      ];
      for (var i in simple_params) {
        let key = simple_params[i];
        if (this[key]) {
          params[key] = this[key];
        }
      }
      if (this.arrayPosition == "stream") {
        params["json_lines"] = true;
      }
      if (this.arrayPosition == "top") {
        params["array_key"] = "";
      }
      return params;
    },
    generateDownload(downloadType) {
      let params = this.dataToParams();
      params.id = this.id;
      params.output_format = downloadType;
      let urlParams = new URLSearchParams(params).toString();

      return "/api/convert?" + urlParams;
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
      this.formState = "submitted";
      fetch("/api/convert?" + urlParams, requestData).then(
        function (response) {
          if (response.status != 200) {
            this.apiStatus = response.status;
            response.json().then(
              function (data) {
                this.apiError = data.error;
                this.id = data.id;
                this.fileStart = data.start;
                this.$nextTick(() => {
                  document.getElementById("error").scrollIntoView();
                });
              }.bind(this)
            );
          } else {
            this.apiStatus = 200;
            response.json().then(
              function (data) {
                this.apiResponse = data;
                this.id = data.id;
                this.$nextTick(() => {
                  document.getElementById("success").scrollIntoView();
                });
              }.bind(this)
            );
          }
        }.bind(this)
      );
    },
    uploadFormData(uploadFile) {
      let formData = new FormData();
      if (uploadFile) {
        formData.append("file", this.fileUpload);
      }
      if (this.fieldsUpload) {
        formData.append("fields", this.fieldsUpload);
      }
      if (this.tablesUpload) {
        formData.append("tables", this.tablesUpload);
      }
      return formData;
    },
    upload(params) {
      let urlParams = new URLSearchParams(params).toString();
      let formData = this.uploadFormData(true);
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
      let formData = this.uploadFormData(false);
      let requestData = {
        headers: {},
        body: formData,
      };
      this.postToApi(urlParams, requestData);
      this.submitType = "url";
    },
    submitPaste(params) {
      let urlParams = new URLSearchParams(params).toString();
      let requestData = {
        headers: {
          "Content-Type": "application/json",
        },
        body: this.paste,
      };
      this.postToApi(urlParams, requestData);
      this.submitType = "paste";
    },
    onIntersect(isIntersecting, entries) {
      if (entries[0].isIntersecting) {
        this.$store.commit("setListItem", entries[0].target.id);
      }
    },
    uploadFile(e, upload_type) {
      if (e.target.files && e.target.files.length > 0) {
        this[upload_type] = e.target.files[0]
      } else {
        this[upload_type] = null
      }
    }
  },
};
</script>
