<script lang="ts">
	import Loading from '$lib/Loading.svelte';
	import type { DatasetViewContext } from '$lib/types';
	import type { MetadataBlock } from '$lib/kamu';
	import { getContext } from 'svelte';
	import { getClient, gql } from '$lib/gql';

	const ctx: DatasetViewContext = getContext('dataset_view');

	let blocks: ArrayLike<MetadataBlock> = null;

	getClient()
		.query({
			query: gql`
				query getDatasetMetadata($dataset_id: String) {
					datasets {
						byId(id: $dataset_id) {
							metadata {
								chain {
									blocks {
										edges {
											node {
												blockHash
												systemTime
											}
										}
									}
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
			blocks = result.data.datasets.byId.metadata.chain.blocks.edges.map((e) => {
				return e.node;
			});
		})
		.catch((reason) => {
			console.log('Gql request failed:', reason);
		});
</script>

{#if blocks == null}
	<Loading what="dataset metadata" />
{:else}
	<h3>Metadata Chain</h3>
	<ul>
		{#each blocks as block}
			<li>
				<a href="?">{block.blockHash}</a>
				<div>{block.systemTime}</div>
			</li>
		{/each}
	</ul>
{/if}
