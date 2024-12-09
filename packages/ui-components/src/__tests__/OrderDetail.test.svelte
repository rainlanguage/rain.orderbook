<script lang="ts">
	import TanstackPageContentDetail from '../lib/components/detail/TanstackPageContentDetail.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import type { Order } from '@rainlanguage/orderbook/js_api';
	import { getOrder } from '@rainlanguage/orderbook/js_api';
	import { QKEY_ORDER } from '../lib/queries/keys';
	import type { Readable } from 'svelte/store';
	import { Button } from 'flowbite-svelte';

	export let walletAddressMatchesOrBlank: Readable<(address: string) => boolean> | undefined =
		undefined;
	export let handleOrderRemoveModal: ((order: Order, refetch: () => void) => void) | undefined =
		undefined;
	export let id: string;
	export let subgraphUrl: string;

	$: orderDetailQuery = createQuery<Order>({
		queryKey: [id, QKEY_ORDER + id],
		queryFn: () => getOrder(subgraphUrl, id),
		enabled: !!subgraphUrl && !!id
	});
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
	<svelte:fragment slot="top" let:data>
		<div>Order {data.orderHash}</div>
		{#if data && $walletAddressMatchesOrBlank?.(data.owner) && data.active && handleOrderRemoveModal}
			<Button
				data-testid="remove-button"
				color="dark"
				on:click={() => handleOrderRemoveModal(data, $orderDetailQuery.refetch)}
				disabled={!handleOrderRemoveModal}
			>
				Remove
			</Button>
		{/if}
	</svelte:fragment>

	<svelte:fragment slot="card" let:data>
		<div>Owner: {data.owner}</div>
	</svelte:fragment>

	<svelte:fragment slot="chart">
		<div>Chart placeholder</div>
	</svelte:fragment>

	<svelte:fragment slot="below" let:data>
		<div>Below content: {data.id}</div>
	</svelte:fragment>
</TanstackPageContentDetail>
