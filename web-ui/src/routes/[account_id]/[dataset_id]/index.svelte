<script lang="ts">
	import { getClient, gql } from '$lib/gql';
	import Loading from '$lib/Loading.svelte';
	import type { Dataset, DatasetViewContext } from '$lib/types';
	import { getContext } from 'svelte';

	const ctx: DatasetViewContext = getContext('dataset_view');

	let dataset: Dataset = null;

	getClient()
		.query({
			query: gql`
				query getDatasetDetails($dataset_id: String) {
					datasets {
						byId(id: $dataset_id) {
							id
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
		})
		.catch((reason) => {
			console.log('Gql request failed:', reason);
		});
</script>

<h1>{ctx.account_id}/{ctx.dataset_id}</h1>

{#if dataset == null}
	<Loading what="dataset" />
{:else}
	<!-- Metadata summary -->
	<ul>
		<li><b>Owner:</b> <span>{ctx.account_id}</span></li>
		<li><b>License:</b> <span>-</span></li>
		<li><b>Last Updated:</b> <span>-</span></li>
		<li><b>Created:</b> <span>-</span></li>
	</ul>

	<!-- Data links -->
	<ul>
		<li><b>Kamu CLI:</b> <span>kamu pull {ctx.account_id}/{ctx.dataset_id}</span></li>
		<li><a href="?">Download Zip</a></li>
	</ul>
{/if}
