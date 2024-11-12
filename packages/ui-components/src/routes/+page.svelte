<script lang="ts">
	import OrdersListTable from '$lib/components/OrdersListTable.svelte';
	import networkSubgraphJson from '$lib/network-subgraph-config.json';
	import { activeSubgraphs, settings } from '$lib/stores/settings';

	$: settings.set(networkSubgraphJson);
	$: activeSubgraphs.set(networkSubgraphJson.subgraphs);
	let connectedWalletAddress: string = '0xf08bCbce72f62c95Dcb7c07dCb5Ed26ACfCfBc11';
</script>

<h1>Welcome to your library project</h1>
<p>Create your package using @sveltejs/package and preview/showcase your work with SvelteKit</p>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>

<OrdersListTable {connectedWalletAddress}>
	<svelte:fragment slot="filters"
		><div class="border-4 border-red-500 p-2 text-xl">Place for filters</div></svelte:fragment
	>
</OrdersListTable>
