<script lang="ts">
	import { getClient, gql } from '$lib/gql';
	import Loading from '$lib/Loading.svelte';
	import type { Dataset, DatasetViewContext } from '$lib/types';
	import { getContext } from 'svelte';

	const ctx: DatasetViewContext = getContext('dataset_view');

	let dataset: Dataset = null;

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
							numRecordsTotal
							currentWatermark
							dataSize

							currentSchema(format: "PARQUET_JSON") {
								format
								content
							}

							tail(numRecords: 20, format: "JSON") {
								format
								content
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
			let d = Object.assign({}, result.data.datasets.byId);

			d.currentSchema = Object.assign({}, d.currentSchema);
			d.currentSchema.content = JSON.parse(d.currentSchema.content);

			d.tail = Object.assign({}, d.tail);
			d.tail.content = JSON.parse(d.tail.content);

			dataset = d;
		})
		.catch((reason) => {
			console.log('Gql request failed:', reason);
		});
</script>

<h1>{ctx.account_id}/{ctx.dataset_id}</h1>

{#if dataset == null}
	<Loading what="dataset" />
{:else}
	<h3>Metadata</h3>
	<ul>
		<li><b>Owner:</b> <span>{ctx.account_id}</span></li>
		<li><b>License:</b> <span>-</span></li>
		<li><b>Last Updated:</b> <span>{dataset.lastUpdatedAt}</span></li>
		<li><b>Created:</b> <span>{dataset.createdAt}</span></li>
		<li><b>Records:</b> <span>{dataset.numRecordsTotal}</span></li>
		<li><b>Estimated Size:</b> <span>{dataset.dataSize} B</span></li>
		<li><b>Watermark:</b> <span>{dataset.currentWatermark}</span></li>
	</ul>

	<h3>Schema</h3>
	<pre>{JSON.stringify(dataset.currentSchema.content, null, 2)}</pre>

	<h3>Data</h3>
	<pre>{JSON.stringify(dataset.tail.content, null, 2)}</pre>

	<h3>Get Data</h3>
	<ul>
		<li>
			<b>Kamu CLI:</b>
			<pre>kamu pull {ctx.account_id}/{ctx.dataset_id}</pre>
		</li>
		<li><b>Download as:</b> <a href="?">CSV</a> | <a href="?">Parquet</a></li>
	</ul>
{/if}
