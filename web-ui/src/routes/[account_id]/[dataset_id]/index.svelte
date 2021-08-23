<script context="module" lang="ts">
	/**
	 * @type {import('@sveltejs/kit').Load}
	 */
	export async function load({ page }) {
		return {
			props: {
				account_id: page.params.account_id,
				dataset_id: page.params.dataset_id
			}
		};
	}
</script>

<script lang="ts">
	import { getClient, gql } from '$lib/gql';
	import Loading from '$lib/Loading.svelte';
	import type { Dataset } from '$lib/types';

	export let account_id: String;
	export let dataset_id: String;

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
				dataset_id
			}
		})
		.then((result) => {
			dataset = result.data.datasets.byId;
		})
		.catch((reason) => {
			console.log('Gql request failed:', reason);
		});
</script>

<a href="/{account_id}/{dataset_id}">Overview</a>
<a href="/{account_id}/{dataset_id}">Data</a>
<a href="/{account_id}/{dataset_id}">Metadata</a>
<a href="/{account_id}/{dataset_id}">Lineage</a>
<a href="/{account_id}/{dataset_id}">Projections</a>
<a href="/{account_id}/{dataset_id}">Issues</a>

<h1>{account_id}/{dataset_id}</h1>

{#if dataset == null}
	<Loading what="dataset" />
{:else}
	<ul>
		<li><b>Owner:</b> <span>{account_id}</span></li>
		<li><b>Created:</b> <span>???</span></li>
		<li><b>Last Updated:</b> <span>???</span></li>
	</ul>
{/if}
