<script lang="ts" generics="T">
	import { toHex } from 'viem';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';

	import {
		Button,
		Dropdown,
		DropdownItem,
		TableBodyCell,
		TableHeadCell,
		Checkbox,
		Tooltip
	} from 'flowbite-svelte';
	import { goto } from '$app/navigation';
	import { DotsVerticalOutline } from 'flowbite-svelte-icons';
	import { createInfiniteQuery, createQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import ListViewOrderbookFilters from '../ListViewOrderbookFilters.svelte';
	import OrderOrVaultHash from '../OrderOrVaultHash.svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '../../queries/constants';
	import { RaindexClient, RaindexVault } from '@rainlanguage/orderbook';
	import { QKEY_TOKENS, QKEY_VAULTS } from '../../queries/keys';
	import type { AppStoresInterface } from '$lib/types/appStores.ts';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { getNetworkName } from '$lib/utils/getNetworkName';
	import { getAllContexts } from 'svelte';

	const context = getAllContexts();

	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'];
	export let orderHash: AppStoresInterface['orderHash'];
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];
	export let activeTokens: AppStoresInterface['activeTokens'];
	export let selectedChainIds: AppStoresInterface['selectedChainIds'];
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
	export let onWithdrawMultiple:
		| ((raindexClient: RaindexClient, vaults: RaindexVault[]) => void)

	const { account, matchesAccount } = useAccount();
	const raindexClient = useRaindexClient();

	// State for selected vaults for multiple withdrawal
	let selectedVaults: RaindexVault[] = [];

	// Helper functions for vault selection
	const isVaultSelected = (vault: RaindexVault): boolean => {
		return selectedVaults.some((v) => v.id === vault.id);
	};

	const isVaultEmpty = (vault: RaindexVault): boolean => {
		return vault.balance === 0n;
	};

	const isVaultDisabled = (vault: RaindexVault): boolean => {
		if (isVaultEmpty(vault)) return true;
		if (!matchesAccount(vault.owner)) return true; // Only allow selection of user's own vaults
		if (selectedVaults.length === 0) return false;
		return vault.chainId !== selectedVaults[0].chainId;
	};

	const toggleVaultSelection = (vault: RaindexVault): void => {
		if (isVaultDisabled(vault)) return;

		if (isVaultSelected(vault)) {
			selectedVaults = selectedVaults.filter((v) => v.id !== vault.id);
		} else {
			selectedVaults = [...selectedVaults, vault];
		}
	};

	const handleWithdrawAll = async () => {
		if (onWithdrawMultiple) {
			await onWithdrawMultiple(raindexClient, selectedVaults);
			selectedVaults = []; // Clear selection after withdrawal
		}
	};

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

	$: query = createInfiniteQuery({
		queryKey: [QKEY_VAULTS, $hideZeroBalanceVaults, $selectedChainIds, owners, selectedTokens],
		queryFn: async ({ pageParam }) => {
			const result = await raindexClient.getVaults(
				$selectedChainIds,
				{
					owners,
					hideZeroBalance: $hideZeroBalanceVaults,
					tokens: selectedTokens
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

	const AppTable = TanstackAppTable<RaindexVault>;
</script>

{#if $query}
	<ListViewOrderbookFilters
		{selectedChainIds}
		{activeAccountsItems}
		{showMyItemsOnly}
		{showInactiveOrders}
		{orderHash}
		{hideZeroBalanceVaults}
		{activeTokens}
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
					{#if onWithdrawMultiple && $account}
						<Button
							color="alternative"
							disabled={selectedVaults.length === 0}
							class={selectedVaults.length === 0 ? 'text-gray-400' : ''}
							on:click={handleWithdrawAll}
						>
							Withdraw all ({selectedVaults.length})
						</Button>
					{/if}
				</div>
			</div>
		</svelte:fragment>
		<svelte:fragment slot="head">
			{#if onWithdrawMultiple && $account}
				<TableHeadCell padding="px-2 py-4">Select</TableHeadCell>
			{/if}
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
			{#if onWithdrawMultiple && $account}
				{#if matchesAccount(item.owner)}
					<TableBodyCell tdClass="px-2 py-4">
						<div class="relative">
							<Checkbox
								checked={isVaultSelected(item)}
								disabled={isVaultDisabled(item)}
								on:click={(e) => e.stopPropagation()}
								on:change={() => toggleVaultSelection(item)}
								class="cursor-pointer"
							/>
							{#if isVaultEmpty(item)}
								<Tooltip class="w-auto text-xs" placement="top">Vault is empty</Tooltip>
							{/if}
						</div>
					</TableBodyCell>
				{:else}
					<TableBodyCell tdClass="px-2 py-4">
						<!-- Empty cell for alignment when user is not the owner -->
					</TableBodyCell>
				{/if}
			{/if}
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
