<script lang="ts">
	import type { Dataset } from '$lib/kamu';

	export let datasets: ArrayLike<Dataset>;
</script>

<ul>
	{#each datasets as ds}
		<li>
			<a href="/anonymous/{ds.id}">{ds.id}</a><br />
			<span>kind: {ds.kind}</span><br />
			<span>created: {ds.createdAt}</span><br />
			<span>last updated: {ds.lastUpdatedAt}</span>
		</li>
	{/each}
</ul>
