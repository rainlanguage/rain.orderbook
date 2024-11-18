<script lang="ts" generics="T">
	import { QKEY_ORDERS } from '../queries/keys.ts';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import { getOrders, type MultiSubgraphArgs } from '@rainlanguage/orderbook/js_api';
	import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '../queries/constants.ts';
	import TanstackAppTable from './TanstackAppTable.svelte';
	// import { goto } from '$app/navigation';
	import { activeSubgraphs, activeAccounts, activeOrderStatus } from '../stores/settings.ts';
	// import { formatTimestampSecondsAsLocal } from '../utils/time';
	// import { handleOrderRemoveModal } from '../services/modal';
	// import { get } from 'svelte/store';
	import { orderHash } from '../stores/settings.ts';
	import { goto } from '$app/navigation';
	import { Badge, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import { formatTimestampSecondsAsLocal } from '../utils/time.ts';
	import Hash, { HashType } from './Hash.svelte';

	export let connectedWalletAddress: string = '';

	const multiSubgraphArgs: MultiSubgraphArgs[] = Object.entries($activeSubgraphs).map(
		([name, url]) => ({
			url,
			name
		})
	) as MultiSubgraphArgs[];

	$: query = createInfiniteQuery({
		queryKey: [QKEY_ORDERS, $activeAccounts, $activeOrderStatus, $orderHash, $activeSubgraphs],
		queryFn: ({ pageParam }) => {
			return getOrders(
				multiSubgraphArgs,
				{
					owners: [connectedWalletAddress],
					active: true,
					orderHash: undefined
				},
				{ page: pageParam + 1, pageSize: DEFAULT_PAGE_SIZE }
			);
		},
		initialPageParam: 0,
		getNextPageParam(lastPage, _allPages, lastPageParam) {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		},
		refetchInterval: DEFAULT_REFRESH_INTERVAL,
		enabled: Object.keys($activeSubgraphs).length > 0
	});

	$: console.log($query.data);
</script>

{#if $query.data}
	<TanstackAppTable
		{query}
		emptyMessage="No Orders Found"
		on:clickRow={(e) => {
			goto(`/orders/${e.detail.item.order.id}`);
		}}
	>
		<svelte:fragment slot="title">
			<slot name="filters" />
		</svelte:fragment>

		<svelte:fragment slot="head">
			<TableHeadCell data-testid="orderListHeadingNetwork" padding="p-4">Network</TableHeadCell>
			<TableHeadCell data-testid="orderListHeadingActive" padding="p-4">Active</TableHeadCell>
			<TableHeadCell data-testid="orderListHeadingID" padding="p-4">Order</TableHeadCell>
			<TableHeadCell data-testid="orderListHeadingOwner" padding="p-4">Owner</TableHeadCell>
			<TableHeadCell data-testid="orderListHeadingOrderbook" padding="p-4">Orderbook</TableHeadCell>
			<TableHeadCell data-testid="orderListHeadingLastAdded" padding="p-4">Last Added</TableHeadCell
			>
			<TableHeadCell data-testid="orderListHeadingInputs" padding="px-2 py-4"
				>Input Token(s)</TableHeadCell
			>
			<TableHeadCell data-testid="orderListHeadingOutputs" padding="px-2 py-4"
				>Output Token(s)</TableHeadCell
			>
			<TableHeadCell data-testid="orderListHeadingTrades" padding="px-2 py-4">Trades</TableHeadCell>
			<TableHeadCell padding="px-4 py-4"></TableHeadCell>
		</svelte:fragment>

		<svelte:fragment slot="bodyRow" let:item>
			<TableBodyCell data-testid="orderListRowNetwork" tdClass="px-4 py-2">
				{item.subgraphName}
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowActive" tdClass="px-4 py-2">
				{#if item.order.active}
					<Badge color="green">Active</Badge>
				{:else}
					<Badge color="yellow">Inactive</Badge>
				{/if}
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowID" tdClass="break-all px-4 py-4">
				<Hash type={HashType.Identifier} value={item.order.orderHash} />
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowOwner" tdClass="break-all px-4 py-2">
				<Hash type={HashType.Wallet} value={item.order.owner} />
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowOrderbook" tdClass="break-all px-4 py-2">
				<Hash type={HashType.Identifier} value={item.order.orderbook.id} />
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowLastAdded" tdClass="break-word px-4 py-2">
				{formatTimestampSecondsAsLocal(BigInt(item.order.timestampAdded))}
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowInputs" tdClass="break-word p-2">
				{item.order.inputs?.map((t) => t.token.symbol)}
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowOutputs" tdClass="break-word p-2">
				{item.order.outputs?.map((t) => t.token.symbol)}
			</TableBodyCell>
			<TableBodyCell data-testid="orderListRowTrades" tdClass="break-word p-2"
				>{item.order.trades.length > 99 ? '>99' : item.order.trades.length}</TableBodyCell
			>
		</svelte:fragment>
	</TanstackAppTable>
{/if}
