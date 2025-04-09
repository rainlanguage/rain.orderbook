<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import Hash, { HashType } from './Hash.svelte';
	import type { SgOrderAsIO, SgOrder, SgVault } from '@rainlanguage/orderbook';
	import {
		constructHashLink,
		isOrderOrVaultActive,
		extractHash
	} from '$lib/utils/constructHashLink';

	type OrderOrVault = SgOrder | SgOrderAsIO | SgVault;

	export let orderOrVault: OrderOrVault;
	export let type: 'orders' | 'vaults';
	export let network: string;
	export let updateActiveNetworkAndOrderbook: (subgraphName: string) => void;

	$: hash = extractHash(orderOrVault);
	$: isActive = isOrderOrVaultActive(orderOrVault);
	$: linkPath = constructHashLink(orderOrVault, type, network);
</script>

<a data-testid="order-or-vault-hash" href={linkPath}>
	<Button
		class="mr-1 mt-1 px-2 py-1 text-sm"
		color={isActive ? 'green' : 'yellow'}
		data-testid="vault-order-input"
		data-id={hash}
		on:click={() => {
			updateActiveNetworkAndOrderbook(network);
		}}><Hash type={HashType.Identifier} value={hash} copyOnClick={false} /></Button
	>
</a>
