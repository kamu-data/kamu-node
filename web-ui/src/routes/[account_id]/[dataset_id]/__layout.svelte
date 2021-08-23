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
	import { setContext } from 'svelte';
	import type { DatasetViewContext } from '$lib/types';

	export let account_id: String;
	export let dataset_id: String;

	const ctx: DatasetViewContext = { account_id, dataset_id };
	setContext('dataset_view', ctx);
</script>

<a href="/{account_id}/{dataset_id}">Overview</a>
<a href="/{account_id}/{dataset_id}">Data</a>
<a href="/{account_id}/{dataset_id}">Metadata</a>
<a href="/{account_id}/{dataset_id}/lineage">Lineage</a>
<a href="/{account_id}/{dataset_id}">Projections</a>
<a href="/{account_id}/{dataset_id}">Issues</a>

<slot />
