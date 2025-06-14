<script lang="ts" generics="T">
	import { Button, Dropdown, DropdownItem, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import { goto } from '$app/navigation';
	import { DotsVerticalOutline } from 'flowbite-svelte-icons';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import ListViewOrderbookFilters from '../ListViewOrderbookFilters.svelte';
	import OrderOrVaultHash from '../OrderOrVaultHash.svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '../../queries/constants';
	import { vaultBalanceDisplay } from '../../utils/vault';
	import { bigintStringToHex } from '../../utils/hex';
	import { type SgVault } from '@rainlanguage/orderbook';
	import { QKEY_VAULTS } from '../../queries/keys';
	import { getVaults, type SgVaultWithSubgraphName } from '@rainlanguage/orderbook';
	import type { AppStoresInterface } from '$lib/types/appStores.ts';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { getMultiSubgraphArgs } from '$lib/utils/configHelpers';

	export let activeOrderbook: AppStoresInterface['activeOrderbook'];
	export let accounts: AppStoresInterface['accounts'];
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'];
	export let orderHash: AppStoresInterface['orderHash'];
	export let activeSubgraphs: AppStoresInterface['activeSubgraphs'];
	export let settings: AppStoresInterface['settings'];
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];
	export let activeNetworkRef: AppStoresInterface['activeNetworkRef'];
	export let activeOrderbookRef: AppStoresInterface['activeOrderbookRef'];
	export let activeAccounts: AppStoresInterface['activeAccounts'];

	export let handleDepositGenericModal: (() => void) | undefined = undefined;
	export let handleDepositModal: ((vault: SgVault, refetch: () => void) => void) | undefined =
		undefined;
	export let handleWithdrawModal: ((vault: SgVault, refetch: () => void) => void) | undefined =
		undefined;
	export let showMyItemsOnly: AppStoresInterface['showMyItemsOnly'];

	const { account, matchesAccount } = useAccount();

	$: multiSubgraphArgs = getMultiSubgraphArgs(
		Object.keys($activeSubgraphs).length > 0 ? $activeSubgraphs : $settings.orderbook.subgraphs
	);

	$: owners =
		$activeAccountsItems && Object.values($activeAccountsItems).length > 0
			? Object.values($activeAccountsItems)
			: $showMyItemsOnly && $account
				? [$account]
				: [];
	$: query = createInfiniteQuery({
		queryKey: [
			QKEY_VAULTS,
			$activeAccounts,
			$hideZeroBalanceVaults,
			$activeSubgraphs,
			multiSubgraphArgs,
			$settings,
			owners
		],
		queryFn: async ({ pageParam }) => {
			const result = await getVaults(
				multiSubgraphArgs,
				{
					owners,
					hideZeroBalance: $hideZeroBalanceVaults
				},
				{ page: pageParam + 1, pageSize: DEFAULT_PAGE_SIZE }
			);
			if (result.error) throw new Error(result.error.msg);
			return result.value;
		},
		initialPageParam: 0,
		getNextPageParam(lastPage, _allPages, lastPageParam) {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		},
		refetchInterval: DEFAULT_REFRESH_INTERVAL,
		enabled: true
	});

	const updateActiveNetworkAndOrderbook = (subgraphName: string) => {
		activeNetworkRef.set(subgraphName);
		activeOrderbookRef.set(subgraphName);
	};
	const AppTable = TanstackAppTable<SgVaultWithSubgraphName>;
</script>

{#if $query}
	<ListViewOrderbookFilters
		{activeSubgraphs}
		{settings}
		{accounts}
		{activeAccountsItems}
		{showMyItemsOnly}
		{showInactiveOrders}
		{orderHash}
		{hideZeroBalanceVaults}
	/>
	<AppTable
		{query}
		queryKey={QKEY_VAULTS}
		emptyMessage="No Vaults Found"
		on:clickRow={(e) => {
			updateActiveNetworkAndOrderbook(e.detail.item.subgraphName);
			goto(`/vaults/${e.detail.item.subgraphName}-${e.detail.item.vault.id}`);
		}}
	>
		<svelte:fragment slot="title">
			<div class="mt-2 flex w-full justify-between">
				<div class="flex items-center gap-x-6">
					<div class="text-3xl font-medium dark:text-white">Vaults</div>
					{#if handleDepositGenericModal}
						<Button
							disabled={!$activeOrderbook}
							size="sm"
							color="primary"
							data-testid="new-vault-button"
							on:click={() => {
								handleDepositGenericModal();
							}}
							>New vault
						</Button>
					{/if}
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
				{item.subgraphName}
			</TableBodyCell>

			<TableBodyCell tdClass="break-all px-4 py-4" data-testid="vault-id">
				<Hash type={HashType.Identifier} value={bigintStringToHex(item.vault.vaultId)} />
			</TableBodyCell>
			<TableBodyCell tdClass="break-all px-4 py-2 min-w-48" data-testid="vault-orderbook">
				<Hash type={HashType.Identifier} value={item.vault.orderbook.id} />
			</TableBodyCell>
			<TableBodyCell tdClass="break-all px-4 py-2 min-w-48" data-testid="vault-owner">
				<Hash type={HashType.Wallet} value={item.vault.owner} />
			</TableBodyCell>
			<TableBodyCell tdClass="break-word p-2 min-w-48" data-testid="vault-token"
				>{item.vault.token.name}</TableBodyCell
			>
			<TableBodyCell tdClass="break-all p-2 min-w-48" data-testid="vault-balance">
				{vaultBalanceDisplay(item.vault)}
				{item.vault.token.symbol}
			</TableBodyCell>
			<TableBodyCell tdClass="break-all p-2 min-w-48">
				{#if item.vault.ordersAsInput.length > 0}
					<div data-testid="vault-order-inputs" class="flex flex-wrap items-end justify-start">
						{#each item.vault.ordersAsInput.slice(0, 3) as order}
							<OrderOrVaultHash
								type="orders"
								orderOrVault={order}
								network={item.subgraphName}
								{updateActiveNetworkAndOrderbook}
							/>
						{/each}
						{#if item.vault.ordersAsInput.length > 3}...{/if}
					</div>
				{/if}
			</TableBodyCell>
			<TableBodyCell tdClass="break-all p-2 min-w-48">
				{#if item.vault.ordersAsOutput.length > 0}
					<div data-testid="vault-order-outputs" class="flex flex-wrap items-end justify-start">
						{#each item.vault.ordersAsOutput.slice(0, 3) as order}
							<OrderOrVaultHash
								type="orders"
								orderOrVault={order}
								network={item.subgraphName}
								{updateActiveNetworkAndOrderbook}
							/>
						{/each}
						{#if item.vault.ordersAsOutput.length > 3}...{/if}
					</div>
				{/if}
			</TableBodyCell>
			{#if handleDepositModal && handleWithdrawModal && matchesAccount(item.vault.owner)}
				<TableBodyCell tdClass="px-0 text-right">
					<Button
						color="alternative"
						outline={false}
						data-testid="vault-menu"
						id={`vault-menu-${item.vault.id}`}
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
					triggeredBy={`#vault-menu-${item.vault.id}`}
				>
					<DropdownItem
						data-testid="deposit-button"
						on:click={(e) => {
							e.stopPropagation();
							handleDepositModal(item.vault, $query.refetch);
						}}
						>Deposit
					</DropdownItem>
					<DropdownItem
						data-testid="withdraw-button"
						on:click={(e) => {
							e.stopPropagation();
							handleWithdrawModal(item.vault, $query.refetch);
						}}
						>Withdraw
					</DropdownItem>
				</Dropdown>
			{/if}
		</svelte:fragment>
	</AppTable>
{/if}
