<script lang="ts" generics="T">
	import { writable } from 'svelte/store';
	import { toHex } from 'viem';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	import { Button, Dropdown, DropdownItem, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import { goto } from '$app/navigation';
	import { DotsVerticalOutline } from 'flowbite-svelte-icons';
	import { createInfiniteQuery, createQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import ListViewOrderbookFilters from '../ListViewOrderbookFilters.svelte';
	import OrderOrVaultHash from '../OrderOrVaultHash.svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '../../queries/constants';
	import { RaindexVault } from '@rainlanguage/orderbook';
	import { QKEY_TOKENS, QKEY_VAULTS } from '../../queries/keys';
	import type { AppStoresInterface } from '$lib/types/appStores.ts';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { getNetworkName } from '$lib/utils/getNetworkName';
	import { useFilterStore } from '$lib/providers/filters';
	import { getAllContexts, onDestroy } from 'svelte';

	const context = getAllContexts();

	// Keep some legacy props that are not filter-related
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'];
	export let orderHash: AppStoresInterface['orderHash'];
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	export let showMyItemsOnly: AppStoresInterface['showMyItemsOnly'];
	export let handleDepositModal:
		| ((
				vault: RaindexVault,
				refetch: () => void,
				context: ReturnType<typeof getAllContexts>
		  ) => void)
		| undefined = undefined;
	export let handleWithdrawModal:
		| ((
				vault: RaindexVault,
				refetch: () => void,
				context: ReturnType<typeof getAllContexts>
		  ) => void)
		| undefined = undefined;

	const { account, matchesAccount } = useAccount();
	const raindexClient = useRaindexClient();

	// Use our new filter store instead of props
	const filterStore = useFilterStore();

	// Get filters from our store
	$: currentFilters = $filterStore?.getVaultsFilters() || {
		owners: [],
		hideZeroBalance: false,
		tokens: undefined,
		chainIds: undefined
	};

	// Derive values from filter store
	$: selectedChainIds = currentFilters.chainIds || [];
	$: hideZeroBalanceVaults = currentFilters.hideZeroBalance;
	$: activeTokens = currentFilters.tokens || [];

	// Create writable stores that sync with our filter store
	// These will be used by ListViewOrderbookFilters for direct updates
	const selectedChainIdsStore = writable<number[]>([]);
	const hideZeroBalanceVaultsStore = writable<boolean>(false);
	const activeTokensStore = writable<`0x${string}`[]>([]);

	// Flag to prevent circular updates during initialization
	let isInitialized = false;

	// Update writable stores when filter store values change (one way sync)
	$: {
		selectedChainIdsStore.set(selectedChainIds);
		hideZeroBalanceVaultsStore.set(hideZeroBalanceVaults);
		activeTokensStore.set(activeTokens);
		// Mark as initialized after first sync
		if (!isInitialized) {
			isInitialized = true;
		}
	}

	const unsubs: (() => void)[] = [];

	// Subscribe to store changes and update filter store accordingly (two way sync)
	// Only after initialization to prevent circular updates
	unsubs.push(
		selectedChainIdsStore.subscribe((chainIds) => {
			if (isInitialized && $filterStore) {
				$filterStore.updateVaults((builder) => builder.setChainIds(chainIds));
				currentFilters = $filterStore.getVaultsFilters();
			}
		})
	);

	unsubs.push(
		hideZeroBalanceVaultsStore.subscribe((hide) => {
			if (isInitialized && $filterStore) {
				$filterStore.updateVaults((builder) => builder.setHideZeroBalance(hide));
				currentFilters = $filterStore.getVaultsFilters();
			}
		})
	);

	unsubs.push(
		activeTokensStore.subscribe((tokens) => {
			if (isInitialized && $filterStore) {
				$filterStore.updateVaults((builder) => builder.setTokens(tokens));
				currentFilters = $filterStore.getVaultsFilters();
			}
		})
	);

	unsubs.push(
		showMyItemsOnly.subscribe((show) => {
			if (isInitialized && $filterStore) {
				if (show && $account) {
					$filterStore.updateVaults((builder) => builder.setOwners([$account]));
				} else {
					$filterStore.updateVaults((builder) => builder.setOwners([]));
				}
				currentFilters = $filterStore.getVaultsFilters();
			}
		})
	);

	onDestroy(() => {
		// Clean up subscriptions
		unsubs.forEach((unsubscribe) => unsubscribe());
	});

	$: tokensQuery = createQuery({
		queryKey: [QKEY_TOKENS, selectedChainIds],
		queryFn: async () => {
			const result = await raindexClient.getAllVaultTokens(selectedChainIds);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		},
		enabled: true
	});

	$: selectedTokens =
		activeTokens?.filter(
			(address: string) =>
				!$tokensQuery.data || $tokensQuery.data.some((t) => t.address === address)
		) ?? [];

	$: query = createInfiniteQuery({
		queryKey: [QKEY_VAULTS, currentFilters],
		queryFn: async ({ pageParam }) => {
			const result = await raindexClient.getVaults(currentFilters, pageParam + 1);
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

	const AppTable = TanstackAppTable<RaindexVault>;
</script>

{#if $query}
	<ListViewOrderbookFilters
		selectedChainIds={selectedChainIdsStore}
		{activeAccountsItems}
		{showMyItemsOnly}
		{showInactiveOrders}
		{orderHash}
		hideZeroBalanceVaults={hideZeroBalanceVaultsStore}
		activeTokens={activeTokensStore}
		{tokensQuery}
		{selectedTokens}
	/>
	<AppTable
		{query}
		queryKey={QKEY_VAULTS}
		emptyMessage="No Vaults Found"
		on:clickRow={(e) => {
			goto(`/vaults/${e.detail.item.chainId}-${e.detail.item.orderbook}-${e.detail.item.id}`);
		}}
	>
		<svelte:fragment slot="title">
			<div class="mt-2 flex w-full justify-between">
				<div class="flex items-center gap-x-6">
					<div class="text-3xl font-medium dark:text-white">Vaults</div>
				</div>
			</div>
		</svelte:fragment>
		<svelte:fragment slot="head">
			<TableHeadCell padding="p-4">Network</TableHeadCell>
			<TableHeadCell padding="px-4 py-4">Vault ID</TableHeadCell>
			<TableHeadCell padding="px-4 py-4">Orderbook</TableHeadCell>
			<TableHeadCell padding="px-4 py-4">Owner</TableHeadCell>
			<TableHeadCell padding="px-2 py-4">Token</TableHeadCell>
			<TableHeadCell padding="px-2 py-4">Balance</TableHeadCell>
			<TableHeadCell padding="px-3 py-4">Input For</TableHeadCell>
			<TableHeadCell padding="px-3 py-4">Output For</TableHeadCell>
		</svelte:fragment>

		<svelte:fragment slot="bodyRow" let:item>
			<TableBodyCell tdClass="px-4 py-2" data-testid="vault-network">
				{getNetworkName(Number(item.chainId))}
			</TableBodyCell>

			<TableBodyCell tdClass="break-all px-4 py-4" data-testid="vault-id">
				<Hash type={HashType.Identifier} value={toHex(item.vaultId)} />
			</TableBodyCell>
			<TableBodyCell tdClass="break-all px-4 py-2 min-w-48" data-testid="vault-orderbook">
				<Hash type={HashType.Identifier} value={item.orderbook} />
			</TableBodyCell>
			<TableBodyCell tdClass="break-all px-4 py-2 min-w-48" data-testid="vault-owner">
				<Hash type={HashType.Wallet} value={item.owner} />
			</TableBodyCell>
			<TableBodyCell tdClass="break-word p-2 min-w-48" data-testid="vault-token"
				>{item.token.name}</TableBodyCell
			>
			<TableBodyCell tdClass="break-all p-2 min-w-48" data-testid="vault-balance">
				{`${item.formattedBalance} ${item.token.symbol}`}
			</TableBodyCell>
			<TableBodyCell tdClass="break-all p-2 min-w-48">
				{#if item.ordersAsInput.length > 0}
					<div data-testid="vault-order-inputs" class="flex flex-wrap items-end justify-start">
						{#each item.ordersAsInput.slice(0, 3) as order}
							<OrderOrVaultHash
								type="orders"
								orderOrVault={order}
								chainId={item.chainId}
								orderbookAddress={item.orderbook}
							/>
						{/each}
						{#if item.ordersAsInput.length > 3}...{/if}
					</div>
				{/if}
			</TableBodyCell>
			<TableBodyCell tdClass="break-all p-2 min-w-48">
				{#if item.ordersAsOutput.length > 0}
					<div data-testid="vault-order-outputs" class="flex flex-wrap items-end justify-start">
						{#each item.ordersAsOutput.slice(0, 3) as order}
							<OrderOrVaultHash
								type="orders"
								orderOrVault={order}
								chainId={item.chainId}
								orderbookAddress={item.orderbook}
							/>
						{/each}
						{#if item.ordersAsOutput.length > 3}...{/if}
					</div>
				{/if}
			</TableBodyCell>
			{#if handleDepositModal && handleWithdrawModal && matchesAccount(item.owner)}
				<TableBodyCell tdClass="px-0 text-right">
					<Button
						color="alternative"
						outline={false}
						data-testid="vault-menu"
						id={`vault-menu-${item.id}`}
						class="mr-2 border-none px-2"
						on:click={(e) => {
							e.stopPropagation();
						}}
					>
						<DotsVerticalOutline class="dark:text-white" />
					</Button>
				</TableBodyCell>

				<Dropdown
					data-testid="dropdown"
					placement="bottom-end"
					triggeredBy={`#vault-menu-${item.id}`}
				>
					<DropdownItem
						data-testid="deposit-button"
						on:click={(e) => {
							e.stopPropagation();
							handleDepositModal(item, $query.refetch, context);
						}}
						>Deposit
					</DropdownItem>
					<DropdownItem
						data-testid="withdraw-button"
						on:click={(e) => {
							e.stopPropagation();
							handleWithdrawModal(item, $query.refetch, context);
						}}
						>Withdraw
					</DropdownItem>
				</Dropdown>
			{/if}
		</svelte:fragment>
	</AppTable>
{/if}
