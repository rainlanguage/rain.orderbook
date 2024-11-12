<script lang="ts">
	import CardProperty from '$lib/components/CardProperty.svelte';
	import OrdersListTable from '$lib/components/OrdersListTable.svelte';
	import networkSubgraphJson from '$lib/network-subgraph-config.json';
	import { settings } from '$lib/stores/settings.js';

	$: settings.set(networkSubgraphJson);

	$: console.log('Settings: ', $settings);
</script>

<h1>Welcome to your library project</h1>
<p>Create your package using @sveltejs/package and preview/showcase your work with SvelteKit</p>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>
<CardProperty data-testid="vaultDetailVaultId">
	<svelte:fragment slot="key">Vault ID</svelte:fragment>
	<svelte:fragment slot="value">{100000}</svelte:fragment>
</CardProperty>

{#if $settings}
	<OrdersListTable />
{/if}
