<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import Hash, { HashType } from './Hash.svelte';
	import { goto } from '$app/navigation';
	import type { Order, OrderAsIO } from '@rainlanguage/orderbook/js_api';

	export let order: Order | OrderAsIO
	export let subgraphName: string;
	export let updateActiveNetworkAndOrderbook: (subgraphName: string) => void;
</script>

<Button
	class="mr-1 mt-1 px-2 py-1 text-sm"
	color={order.active ? 'green' : 'yellow'}
	data-testid="vault-order-input"
	data-order-id={order.id}
	on:click={() => {
		updateActiveNetworkAndOrderbook(subgraphName);
		goto(`/orders/${subgraphName}-${order.id}`);
	}}><Hash type={HashType.Identifier} value={order.orderHash} copyOnClick={false} /></Button
>
