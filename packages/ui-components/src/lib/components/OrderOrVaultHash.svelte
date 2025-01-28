<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import Hash, { HashType } from './Hash.svelte';
	import { goto } from '$app/navigation';
	import type { OrderAsIO, OrderSubgraph, Vault } from '@rainlanguage/orderbook/js_api';

	type OrderOrVault = OrderSubgraph | OrderAsIO | Vault;

	export let orderOrVault: OrderOrVault;
	export let type: 'orders' | 'vaults';
	export let network: string;
	export let updateActiveNetworkAndOrderbook: (subgraphName: string) => void;

	let hash;

	$: isOrder = 'orderHash' in (orderOrVault || {});
	$: slug = isOrder ? (orderOrVault as OrderSubgraph).id : (orderOrVault as Vault)?.id;
	$: hash = isOrder ? (orderOrVault as OrderSubgraph).id : (orderOrVault as Vault)?.id || '';
	$: isActive = isOrder ? (orderOrVault as OrderAsIO).active : false;
</script>

<Button
	class="mr-1 mt-1 px-2 py-1 text-sm"
	color={isActive ? 'green' : 'yellow'}
	data-testid="vault-order-input"
	data-id={slug}
	on:click={() => {
		updateActiveNetworkAndOrderbook(network);
		goto(`/${type}/${network}-${slug}`);
	}}><Hash type={HashType.Identifier} value={hash} copyOnClick={false} /></Button
>
