<script>
	import CardProperty from '$lib/components/CardProperty.svelte';
</script>

<h1>Welcome to your library project</h1>
<p>Create your package using @sveltejs/package and preview/showcase your work with SvelteKit</p>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>
<CardProperty data-testid="vaultDetailVaultId">
	<svelte:fragment slot="key">Vault ID</svelte:fragment>
	<svelte:fragment slot="value">{100000}</svelte:fragment>
</CardProperty>
