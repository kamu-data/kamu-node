<script lang="ts">
	import DatasetList from '$lib/DatasetList.svelte';
	import Loading from '$lib/Loading.svelte';
	import { getClient, gql } from '$lib/gql';
	import type { Dataset } from '$lib/kamu';

	let search_query = '';

	let pending_promise: Promise<void> = null;
	let dirty = false;

	let datasets: ArrayLike<Dataset> = null;

	function search(query: String) {
		console.log('Sending search query', query);

		pending_promise = getClient()
			.query({
				query: gql`
					query search($query: String) {
						search {
							query(query: $query) {
								edges {
									node {
										__typename
										... on Dataset {
											id
											createdAt
											lastUpdatedAt
										}
									}
								}
							}
						}
					}
				`,
				variables: {
					query: query
				}
			})
			.then((result) => {
				datasets = result.data.search.query.edges.map((edge) => {
					return {
						id: edge.node.id,
						createdAt: edge.node.createdAt,
						lastUpdatedAt: edge.node.lastUpdatedAt
					};
				});
				if (dirty) {
					dirty = false;
					search(search_query);
				}
			})
			.catch((reason) => {
				console.log('Gql request failed:', reason);
			})
			.finally(() => {
				pending_promise = null;
			});
	}

	function onSearchQueryChanged() {
		if (pending_promise != null) {
			dirty = true;
		} else {
			search(search_query);
		}
	}

	search('');
</script>

<svelte:head>
	<title>Kamu</title>
</svelte:head>

<!-- <Search /> -->
<input
	placeholder="Search or jump to..."
	bind:value={search_query}
	on:input={onSearchQueryChanged}
/>
<p>Searching for: {search_query}</p>

{#if datasets == null}
	<Loading what="datasets" />
{:else}
	<DatasetList {datasets} />
{/if}
