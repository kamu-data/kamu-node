<script lang="ts">
	import '../app.css';
	import Search from '$lib/Search.svelte';
</script>

<div class="MainMenu">
	<a href="/">
		<img class="Logo" alt="Kamu Logo" src="/static/kamu_logo_icon.svg" />
	</a>
	<a href="/explore">Explore</a>
	<Search />
</div>

<div class="Content">
	<slot />
</div>

<style>
	.Content {
		padding: 10px;
	}

	.Logo {
		height: 2em;
		padding: 6px;
	}

	.MainMenu {
		background-color: var(--color-bg-menu);
		display: flex;
		flex-wrap: nowrap;
		align-items: center;
	}

	.MainMenu > a {
		padding: 6px;
		color: var(--color-fg-menu);
		text-decoration: none;
		font-size: 14px;
		font-weight: 600;
	}
	.MainMenu > a:hover {
		color: var(--color-fg-menu-alt);
	}
</style>
