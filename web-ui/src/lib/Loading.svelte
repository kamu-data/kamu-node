<script lang="ts">
	export let what = '';
</script>

{#if what}
	<p>Loading {what}...</p>
{:else}
	<p>Loading...</p>
{/if}
