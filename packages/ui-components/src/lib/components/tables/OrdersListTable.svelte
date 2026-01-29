<script lang="ts">
	import { getNetworkName } from '$lib/utils/getNetworkName';
	import { goto } from '$app/navigation';
	import { createInfiniteQuery, createQuery } from '@tanstack/svelte-query';
	import { RaindexOrder, type OrderbookCfg } from '@rainlanguage/orderbook';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { formatTimestampSecondsAsLocal } from '../../services/time';
	import ListViewOrderbookFilters from '../ListViewOrderbookFilters.svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import VaultCard from '../VaultCard.svelte';
	import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '../../queries/constants';
	import { QKEY_ORDERS, QKEY_TOKENS } from '../../queries/keys';
	import type { AppStoresInterface } from '../../types/appStores';
	import { Badge, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	export let selectedChainIds: AppStoresInterface['selectedChainIds'];
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'] | undefined;
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	export let orderHash: AppStoresInterface['orderHash'];
	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];
	export let hideInactiveOrdersVaults: AppStoresInterface['hideInactiveOrdersVaults'];
	export let showMyItemsOnly: AppStoresInterface['showMyItemsOnly'];
	export let activeTokens: AppStoresInterface['activeTokens'];
	export let activeOrderbookAddresses: AppStoresInterface['activeOrderbookAddresses'];

	const { account } = useAccount();
	const raindexClient = useRaindexClient();

	$: owners =
		$activeAccountsItems && Object.values($activeAccountsItems).length > 0
			? Object.values($activeAccountsItems)
			: $showMyItemsOnly && $account
				? [$account]
				: [];

	$: tokensQuery = createQuery({
		queryKey: [QKEY_TOKENS, $selectedChainIds],
		queryFn: async () => {
			const result = await raindexClient.getAllVaultTokens($selectedChainIds);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		},
		enabled: true
	});

	$: selectedTokens =
		$activeTokens?.filter(
			(address) => !$tokensQuery.data || $tokensQuery.data.some((t) => t.address === address)
		) ?? [];

	$: orderbooksMap = raindexClient.getAllOrderbooks()?.value ?? new Map<string, OrderbookCfg>();
	$: availableOrderbookAddresses = (() => {
		const addrs: string[] = [];
		orderbooksMap.forEach((cfg: OrderbookCfg) => {
			if ($selectedChainIds.length === 0 || $selectedChainIds.includes(cfg.network.chainId)) {
				addrs.push(cfg.address.toLowerCase());
			}
		});
		return addrs;
	})();
	$: selectedOrderbookAddresses =
		$activeOrderbookAddresses?.filter((address) =>
			availableOrderbookAddresses.includes(address.toLowerCase())
		) ?? [];

	$: query = createInfiniteQuery({
		queryKey: [
			QKEY_ORDERS,
			$selectedChainIds,
			owners,
			$showInactiveOrders,
			$orderHash,
			selectedTokens,
			selectedOrderbookAddresses
		],
		queryFn: async ({ pageParam }) => {
			const result = await raindexClient.getOrders(
				$selectedChainIds,
				{
					owners,
					active: $showInactiveOrders ? undefined : true,
					orderHash: $orderHash || undefined,
					tokens:
						selectedTokens.length > 0
							? { inputs: selectedTokens, outputs: selectedTokens }
							: undefined,
					orderbookAddresses:
						selectedOrderbookAddresses.length > 0 ? selectedOrderbookAddresses : undefined
				},
				pageParam + 1
			);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		},
		initialPageParam: 0,
		getNextPageParam(lastPage, _allPages, lastPageParam) {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		},
		refetchInterval: DEFAULT_REFRESH_INTERVAL,
		enabled: true
	});

	const AppTable = TanstackAppTable<RaindexOrder>;
</script>

<ListViewOrderbookFilters
	{selectedChainIds}
	{activeAccountsItems}
	{showMyItemsOnly}
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
	{hideInactiveOrdersVaults}
	{tokensQuery}
	{activeTokens}
	{selectedTokens}
	{activeOrderbookAddresses}
	{selectedOrderbookAddresses}
/>

<AppTable
	{query}
	queryKey={QKEY_ORDERS}
	emptyMessage="No Orders Found"
	on:clickRow={(e) => {
		goto(`/orders/${e.detail.item.chainId}-${e.detail.item.orderbook}-${e.detail.item.orderHash}`);
	}}
>
	<svelte:fragment slot="title">
		<div class="mt-2 flex w-full justify-between">
			<div class="text-3xl font-medium dark:text-white" data-testid="title">Orders</div>
			<slot name="filters" />
		</div>
	</svelte:fragment>

	<svelte:fragment slot="head">
		<TableHeadCell data-testid="orderListHeadingOrderInfo" padding="p-4" class="w-[15%]"
			>Order Info</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingAddresses" padding="p-4" class="w-[20%]"
			>Addresses</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingInputs" padding="px-2 py-4" class="w-[27.5%]"
			>Input Token(s)</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingOutputs" padding="px-2 py-4" class="w-[27.5%]"
			>Output Token(s)</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingTrades" padding="px-2 py-4" class="w-[10%]"
			>Trades</TableHeadCell
		>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell data-testid="orderListRowOrderInfo" tdClass="px-4 py-2">
			<div class="flex flex-col gap-1">
				<div class="flex items-center gap-2">
					<span class="text-sm font-medium">{getNetworkName(Number(item.chainId))}</span>
					{#if item.active}
						<Badge color="green">Active</Badge>
					{:else}
						<Badge color="yellow">Inactive</Badge>
					{/if}
				</div>
				<span class="text-xs text-gray-500 dark:text-gray-400">
					Added: {formatTimestampSecondsAsLocal(item.timestampAdded)}
				</span>
			</div>
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowAddresses" tdClass="px-4 py-2">
			<div class="flex flex-col gap-1 text-sm">
				<div class="flex items-center gap-1">
					<span class="text-gray-500 dark:text-gray-400">Order:</span>
					<Hash type={HashType.Identifier} value={item.orderHash} />
				</div>
				<div class="flex items-center gap-1">
					<span class="text-gray-500 dark:text-gray-400">Owner:</span>
					<Hash type={HashType.Wallet} value={item.owner} />
				</div>
				<div class="flex items-center gap-1">
					<span class="text-gray-500 dark:text-gray-400">Orderbook:</span>
					<Hash type={HashType.Identifier} value={item.orderbook} />
				</div>
			</div>
		</TableBodyCell>

		<TableBodyCell data-testid="orderListRowInputs" tdClass="p-2 whitespace-normal">
			<div class="grid w-full grid-cols-1 gap-2 sm:grid-cols-2">
				{#each item.inputsList.items as vault}
					<VaultCard {vault} />
				{/each}
				{#each item.inputsOutputsList.items as vault}
					{#if !item.inputsList.items.find((v) => v.id === vault.id)}
						<VaultCard {vault} />
					{/if}
				{/each}
			</div>
		</TableBodyCell>

		<TableBodyCell data-testid="orderListRowOutputs" tdClass="p-2 whitespace-normal">
			<div class="grid w-full grid-cols-1 gap-2 sm:grid-cols-2">
				{#each item.outputsList.items as vault}
					<VaultCard {vault} />
				{/each}
				{#each item.inputsOutputsList.items as vault}
					{#if !item.outputsList.items.find((v) => v.id === vault.id)}
						<VaultCard {vault} />
					{/if}
				{/each}
			</div>
		</TableBodyCell>

		<TableBodyCell data-testid="orderListRowTrades" tdClass="break-word p-2">
			{item.tradesCount > 99 ? '>99' : item.tradesCount}
		</TableBodyCell>
	</svelte:fragment>
</AppTable>
