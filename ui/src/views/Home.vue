<template>
  <v-container>
		<v-expansion-panels v-model="panel">
			<v-expansion-panel>
				<v-expansion-panel-header>
					Upload
				</v-expansion-panel-header>
				
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
						<v-btn
							color="success"
							@click="upload">
							Upload
						</v-btn>
					</v-form>
				</v-expansion-panel-content>
			</v-expansion-panel>
			<v-expansion-panel>
				<v-expansion-panel-header>
					Link
				</v-expansion-panel-header>
				<v-expansion-panel-content>
					Link
				</v-expansion-panel-content>
			</v-expansion-panel>
			<v-expansion-panel>
				<v-expansion-panel-header>
					Paste
				</v-expansion-panel-header>
				<v-expansion-panel-content>
					Paste
				</v-expansion-panel-content>
			</v-expansion-panel>
		</v-expansion-panels>
  </v-container>
</template>

<script>
  export default {
    name: 'Home',
    data () {
      return {
				panel: 0,
				fileUpload: null,
				fetchResponse: null,
				fetchErorr: null
      }
    },
		methods: {
      upload () {
				console.log(this.fileUpload) 
				let formData = new FormData();

				formData.append( 'file', this.fileUpload );

				fetch('/api/convert?output_format=preview', {
						method: 'POST',
						headers: {
								'Accept': 'application/json',
						},
						body: formData
				} )
				.then( function( response ){
						if( response.status != 201 ){
								this.fetchError = response.status;
						}else{
								response.json().then( function( data ){
										this.fetchResponse = data;
										console.log(data);
								}.bind(this));
						}
				}.bind(this));
      },
		}
  }
</script>
