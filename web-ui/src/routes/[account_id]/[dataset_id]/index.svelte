<script lang="ts">
	import { getClient, gql } from '$lib/gql';
	import Loading from '$lib/Loading.svelte';
	import type { Dataset, DataSchema, DataSlice } from '$lib/kamu';
	import type { DatasetViewContext } from '$lib/types';
	import { getContext } from 'svelte';

	const ctx: DatasetViewContext = getContext('dataset_view');

	let dataset: Dataset = null;
	let schema: any = null;
	let data: any = null;

	function parseSchema(schema: DataSchema) {
		return JSON.parse(schema.content);
	}

	function parseDataSlice(data: DataSlice) {
		return JSON.parse(data.content);
	}

	// TODO: Query data and schema separately to improve page load times
	getClient()
		.query({
			query: gql`
				query getDatasetDetails($dataset_id: String) {
					datasets {
						byId(id: $dataset_id) {
							id
							createdAt
							lastUpdatedAt

							metadata {
								currentWatermark
								currentSchema(format: "PARQUET_JSON") {
									format
									content
								}
							}

							data {
								numRecordsTotal
								estimatedSize

								tail(numRecords: 20, format: "JSON") {
									format
									content
								}
							}
						}
					}
				}
			`,
			variables: {
				dataset_id: ctx.dataset_id
			}
		})
		.then((result) => {
			dataset = result.data.datasets.byId;
			schema = parseSchema(dataset.metadata.currentSchema);
			data = parseDataSlice(dataset.data.tail);
		})
		.catch((reason) => {
			console.log('Gql request failed:', reason);
		});
</script>

{#if dataset == null}
	<Loading what="dataset" />
{:else}
	<h3>Metadata</h3>
	<ul>
		<li><b>Owner:</b> <span>{ctx.account_id}</span></li>
		<li><b>License:</b> <span>-</span></li>
		<li><b>Last Updated:</b> <span>{dataset.lastUpdatedAt}</span></li>
		<li><b>Created:</b> <span>{dataset.createdAt}</span></li>
		<li><b>Records:</b> <span>{dataset.data.numRecordsTotal}</span></li>
		<li><b>Estimated Size:</b> <span>{dataset.data.estimatedSize} B</span></li>
		<li><b>Watermark:</b> <span>{dataset.metadata.currentWatermark}</span></li>
	</ul>

	<h3>Schema</h3>
	<pre>{JSON.stringify(schema, null, 2)}</pre>

	<h3>Data</h3>
	<pre>{JSON.stringify(data, null, 2)}</pre>

	<h3>Get Data</h3>
	<ul>
		<li>
			<b>Kamu CLI:</b>
			<pre>kamu pull {ctx.account_id}/{ctx.dataset_id}</pre>
		</li>
		<li><b>Download as:</b> <a href="?">CSV</a> | <a href="?">Parquet</a></li>
	</ul>
{/if}
