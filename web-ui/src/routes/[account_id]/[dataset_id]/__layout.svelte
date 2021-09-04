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

	export let account_id: string;
	export let dataset_id: string;

	const ctx: DatasetViewContext = { account_id, dataset_id };
	setContext('dataset_view', ctx);
</script>

<a href="/{account_id}/{dataset_id}">Overview</a>
<a href="/{account_id}/{dataset_id}/data">Data</a>
<a href="/{account_id}/{dataset_id}/metadata">Metadata</a>
<a href="/{account_id}/{dataset_id}/lineage">Lineage</a>
<a href="/{account_id}/{dataset_id}">Projections</a>
<a href="/{account_id}/{dataset_id}">Issues</a>

<h1>{ctx.account_id}/{ctx.dataset_id}</h1>
<slot />
