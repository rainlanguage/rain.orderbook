<script lang="ts" generics="T">
	import { goto } from '$app/navigation';
	import { DotsVerticalOutline } from 'flowbite-svelte-icons';
	import { type OrderWithSubgraphName } from '@rainlanguage/orderbook/js_api';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import { getOrders, type MultiSubgraphArgs } from '@rainlanguage/orderbook/js_api';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { formatTimestampSecondsAsLocal } from '../../utils/time';
	import ListViewOrderbookFilters from '../ListViewOrderbookFilters.svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '../../queries/constants';
	import { QKEY_ORDERS } from '../../queries/keys';
	import type { AppStoresInterface } from '../../types/appStores';
	import {
		Badge,
		Button,
		Dropdown,
		DropdownItem,
		TableBodyCell,
		TableHeadCell
	} from 'flowbite-svelte';
	import type { Writable } from 'svelte/store';

	// Optional props only used in tauri-app
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export const walletAddressMatchesOrBlank: any = undefined;
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export const handleOrderRemoveModal: any = undefined;
	// End of optional props

	export let activeSubgraphs: AppStoresInterface['activeSubgraphs'];
	export let settings: AppStoresInterface['settings'];
	export let accounts: AppStoresInterface['accounts'] | undefined;
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'] | undefined;
	export let activeOrderStatus: AppStoresInterface['activeOrderStatus'];
	export let orderHash: AppStoresInterface['orderHash'];
	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];
	export let showMyItemsOnly: AppStoresInterface['showMyItemsOnly'];
	export let currentRoute: string;
	export let signerAddress: Writable<string | null> | undefined;
	export let activeNetworkRef: AppStoresInterface['activeNetworkRef'];
	export let activeOrderbookRef: AppStoresInterface['activeOrderbookRef'];

	$: multiSubgraphArgs = Object.entries(
		Object.keys($activeSubgraphs ?? {}).length ? $activeSubgraphs : ($settings?.subgraphs ?? {})
	).map(([name, url]) => ({
		name,
		url
	})) as MultiSubgraphArgs[];

	$: owners =
		$activeAccountsItems && Object.values($activeAccountsItems).length > 0
			? Object.values($activeAccountsItems)
			: $showMyItemsOnly && $signerAddress
				? [$signerAddress]
				: [];
	$: query = createInfiniteQuery({
		queryKey: [
			QKEY_ORDERS,
			$activeSubgraphs,
			$settings,
			multiSubgraphArgs,
			owners,
			$activeOrderStatus,
			$orderHash
		],
		queryFn: ({ pageParam }) => {
			return getOrders(
				multiSubgraphArgs,
				{
					owners,
					active: $activeOrderStatus,
					orderHash: $orderHash || undefined
				},
				{ page: pageParam + 1, pageSize: DEFAULT_PAGE_SIZE }
			);
		},
		initialPageParam: 0,
		getNextPageParam(lastPage, _allPages, lastPageParam) {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		},
		refetchInterval: DEFAULT_REFRESH_INTERVAL,
		enabled: true
	});

	const AppTable = TanstackAppTable<OrderWithSubgraphName>;

	$: isVaultsPage = currentRoute.startsWith('/vaults');
	$: isOrdersPage = currentRoute.startsWith('/orders');
</script>

<ListViewOrderbookFilters
	{activeSubgraphs}
	{settings}
	{accounts}
	{activeAccountsItems}
	{showMyItemsOnly}
	{activeOrderStatus}
	{orderHash}
	{hideZeroBalanceVaults}
	{isVaultsPage}
	{isOrdersPage}
/>

<AppTable
	{query}
	emptyMessage="No Orders Found"
	on:clickRow={(e) => {
		activeNetworkRef.set(e.detail.item.subgraphName);
		activeOrderbookRef.set(e.detail.item.subgraphName);
		goto(`/orders/${e.detail.item.subgraphName}-${e.detail.item.order.id}`);
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
		<TableHeadCell data-testid="orderListHeadingLastAdded" padding="p-4">Last Added</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingInputs" padding="px-2 py-4"
			>Input Token(s)</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingOutputs" padding="px-2 py-4"
			>Output Token(s)</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingTrades" padding="px-2 py-4">Trades</TableHeadCell>
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
		{#if walletAddressMatchesOrBlank && handleOrderRemoveModal}
			<div data-testid="wallet-actions">
				<TableBodyCell tdClass="px-0 text-right">
					{#if $walletAddressMatchesOrBlank(item.order.owner) && item.order.active}
						<Button
							color="alternative"
							outline={false}
							data-testid={`order-menu-${item.order.id}`}
							id={`order-menu-${item.order.id}`}
							class="mr-2 border-none px-2"
							on:click={(e) => {
								e.stopPropagation();
							}}
						>
							<DotsVerticalOutline class="dark:text-white" />
						</Button>
					{/if}
				</TableBodyCell>
				{#if $walletAddressMatchesOrBlank(item.order.owner) && item.order.active}
					<Dropdown placement="bottom-end" triggeredBy={`#order-menu-${item.order.id}`}>
						<DropdownItem
							on:click={(e) => {
								e.stopPropagation();
								handleOrderRemoveModal(item.order, $query.refetch);
							}}>Remove</DropdownItem
						>
					</Dropdown>
				{/if}
			</div>
		{/if}
	</svelte:fragment>
</AppTable>
