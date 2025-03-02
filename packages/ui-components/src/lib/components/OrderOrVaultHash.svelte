<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import Hash, { HashType } from './Hash.svelte';
	import type { SgOrderAsIO, SgOrder, SgVault } from '@rainlanguage/orderbook/js_api';

	type OrderOrVault = SgOrder | SgOrderAsIO | SgVault;

	export let orderOrVault: OrderOrVault;
	export let type: 'orders' | 'vaults';
	export let network: string;
	export let updateActiveNetworkAndOrderbook: (subgraphName: string) => void;

	let hash;

	$: isOrder = 'orderHash' in (orderOrVault || {});
	$: slug = isOrder ? (orderOrVault as SgOrder).orderHash : (orderOrVault as SgVault)?.id;
	$: hash = isOrder ? (orderOrVault as SgOrder).orderHash : (orderOrVault as SgVault)?.id || '';
	$: isActive = isOrder ? (orderOrVault as SgOrderAsIO).active : false;
</script>

<a href={`/${type}/${network}-${slug}`}>
	<Button
		class="mr-1 mt-1 px-2 py-1 text-sm"
		color={isActive ? 'green' : 'yellow'}
		data-testid="vault-order-input"
		data-id={slug}
		on:click={() => {
			updateActiveNetworkAndOrderbook(network);
		}}><Hash type={HashType.Identifier} value={hash} copyOnClick={false} /></Button
	>
</a>
