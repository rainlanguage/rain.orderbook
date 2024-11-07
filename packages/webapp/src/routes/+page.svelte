<script>
	import { CardProperty } from '@rainlanguage/ui-components';
</script>

<div data-testid="page-container">
	<h1>Welcome to SvelteKit</h1>
	<p>
		Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation...
	</p>

	<CardProperty data-testid="card-property">
		<svelte:fragment slot="key">Vault ID</svelte:fragment>
		<svelte:fragment slot="value">{100000}</svelte:fragment>
	</CardProperty>
</div>
