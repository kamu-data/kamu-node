<script lang="ts">
	import type { Dataset } from '$lib/types';

	export let datasets: ArrayLike<Dataset>;
</script>

<ul>
	{#each datasets as ds}
		<li><a href="/anonymous/{ds.id}">{ds.id}</a></li>
	{/each}
</ul>
