<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import Hash, { HashType } from './Hash.svelte';
	import { goto } from '$app/navigation';
	import type { Order, OrderAsIO, Vault } from '@rainlanguage/orderbook/js_api';

	export let order: Order | OrderAsIO | undefined = undefined;
	export let vault: Vault | undefined = undefined;
	export let type: 'orders' | 'vaults';
	export let subgraphName: string;
	export let updateActiveNetworkAndOrderbook: (subgraphName: string) => void;

	$: id = order?.id || vault?.id;
	$: hash = order?.orderHash || vault?.id;
</script>

{#if hash && id && subgraphName}
	<Button
		class="mr-1 mt-1 px-2 py-1 text-sm"
		color={order?.active ? 'green' : 'yellow'}
		data-testid="vault-order-input"
		data-id={id}
		on:click={() => {
			updateActiveNetworkAndOrderbook(subgraphName);
			goto(`/${type}/${subgraphName}-${id}`);
		}}><Hash type={HashType.Identifier} value={hash} copyOnClick={false} /></Button
	>
{/if}
