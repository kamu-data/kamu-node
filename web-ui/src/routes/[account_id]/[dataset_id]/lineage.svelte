<script lang="ts">
	import Loading from '$lib/Loading.svelte';
	import type { DatasetViewContext } from '$lib/types';
	import { getContext } from 'svelte';

	const ctx: DatasetViewContext = getContext('dataset_view');

	let lineage = null;
</script>

<h1>{ctx.account_id}/{ctx.dataset_id}</h1>

{#if lineage == null}
	<Loading what="dataset lineage" />
{:else}{/if}
