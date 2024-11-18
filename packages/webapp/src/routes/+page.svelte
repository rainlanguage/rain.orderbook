<script lang="ts" generics="T">
	import type { InfiniteQueryObserverResult } from '@tanstack/svelte-query';

	import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
	import { Hash, HashType } from '@rainlanguage/ui-components';
	import { TableHeadCell, TableBodyCell, Badge } from 'flowbite-svelte';
	import { TanstackAppTable } from '@rainlanguage/ui-components';
	import { readable } from 'svelte/store';

	const mockQueryStore = {
		...readable({
			data: {
				pages: [
					[
						{
							// Mocking minimal query structure
							subgraphName: 'Ethereum',
							order: {
								active: true,
								orderHash: '0x1234567890abcdef1234567890abcdef12345678',
								owner: '0xabcdef1234567890abcdef1234567890abcdef12',
								orderbook: { id: '0x9876543210fedcba9876543210fedcba98765432' },
								timestampAdded: '1709251200', // Some timestamp
								inputs: [{ token: { symbol: 'ETH' } }, { token: { symbol: 'USDC' } }],
								outputs: [{ token: { symbol: 'DAI' } }],
								trades: new Array(5)
							}
						}
					]
				]
			}
		}),
		subscribe: () => () => {}
	} as unknown as InfiniteQueryObserverResult;
</script>

<TanstackAppTable query={mockQueryStore}>
	<svelte:fragment slot="title">
		<slot name="filters" />
	</svelte:fragment>

	<svelte:fragment slot="head">
		<TableHeadCell data-testid="orderListHeadingNetwork" padding="p-4">Network</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingActive" padding="p-4">Active</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingID" padding="p-4">Order</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingOwner" padding="p-4">Owner</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingOrderbook" padding="p-4">Orderbook</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingLastAdded" padding="p-4">Last Added</TableHeadCell>
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
