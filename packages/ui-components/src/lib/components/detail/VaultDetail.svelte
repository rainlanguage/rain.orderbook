<script lang="ts">
	import { Button } from 'flowbite-svelte';
	import { bigintStringToHex } from '../../utils/hex';
	import Hash, { HashType } from '../Hash.svelte';
	import VaultBalanceChangesTable from '../tables/VaultBalanceChangesTable.svelte';
	import VaultBalanceChart from '../charts/VaultBalanceChart.svelte';
	import TanstackPageContentDetail from './TanstackPageContentDetail.svelte';
	import CardProperty from '../CardProperty.svelte';
	import { QKEY_VAULT } from '../../queries/keys';
	import { getVault } from '@rainlanguage/orderbook/js_api';
	import type { ChartTheme } from '../../utils/lightweightChartsThemes';
	import { formatUnits } from 'viem';
	import { createQuery } from '@tanstack/svelte-query';

	import { onDestroy } from 'svelte';
	import type { Readable, Writable } from 'svelte/store';
	import { useQueryClient } from '@tanstack/svelte-query';

	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
	import type { Vault } from '@rainlanguage/orderbook/js_api';
	import OrderOrVaultHash from '../OrderOrVaultHash.svelte';
	import type { AppStoresInterface } from '../../types/appStores';
	import type { Config } from 'wagmi';
	import DepositOrWithdrawButtons from './DepositOrWithdrawButtons.svelte';
	import Refresh from '../icon/Refresh.svelte';

	export let handleDepositOrWithdrawModal:
		| ((args: {
				vault: Vault;
				onDepositOrWithdraw: () => void;
				action: 'deposit' | 'withdraw';
				chainId: number;
				rpcUrl: string;
				subgraphUrl: string;
		  }) => void)
		| undefined = undefined;
	export let id: string;
	export let network: string;
	export let walletAddressMatchesOrBlank: Readable<(otherAddress: string) => boolean> | undefined =
		undefined;
	// Tauri App modals
	export let handleDepositModal: ((vault: Vault, onDeposit: () => void) => void) | undefined =
		undefined;
	export let handleWithdrawModal: ((vault: Vault, onWithdraw: () => void) => void) | undefined =
		undefined;

	export let lightweightChartsTheme: Readable<ChartTheme> | undefined = undefined;
	export let activeNetworkRef: AppStoresInterface['activeNetworkRef'];
	export let activeOrderbookRef: AppStoresInterface['activeOrderbookRef'];
	export let settings;
	export let wagmiConfig: Writable<Config> | undefined = undefined;
	export let signerAddress: Writable<string | null> | undefined = undefined;

	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const chainId = $settings?.networks?.[network]?.['chain-id'] || 0;
	const rpcUrl = $settings?.networks?.[network]?.['rpc'] || '';
	const queryClient = useQueryClient();

	$: vaultDetailQuery = createQuery({
		queryKey: [id, QKEY_VAULT + id],
		queryFn: () => {
			return getVault(subgraphUrl || '', id);
		},
		enabled: !!subgraphUrl
	});

	const updateActiveNetworkAndOrderbook = (subgraphName: string) => {
		activeNetworkRef.set(subgraphName);
		activeOrderbookRef.set(subgraphName);
	};

	const interval = setInterval(async () => {
		// This invalidate function invalidates
		// both vault detail and vault balance changes queries
		await queryClient.invalidateQueries({
			queryKey: [id],
			refetchType: 'active',
			exact: false
		});
	}, 5000);

	onDestroy(() => {
		clearInterval(interval);
	});
</script>

<TanstackPageContentDetail query={vaultDetailQuery} emptyMessage="Vault not found">
	<svelte:fragment slot="top" let:data>
		<div
			data-testid="vaultDetailTokenName"
			class="flex gap-x-4 text-3xl font-medium dark:text-white"
		>
			{data.token.name}
		</div>
		<div class="flex gap-2">
			{#if $wagmiConfig && handleDepositOrWithdrawModal && $signerAddress === data.owner}
				<DepositOrWithdrawButtons
					vault={data}
					{chainId}
					{rpcUrl}
					query={vaultDetailQuery}
					{handleDepositOrWithdrawModal}
					{subgraphUrl}
				/>
			{:else if handleDepositModal && handleWithdrawModal && $walletAddressMatchesOrBlank?.(data.owner)}
				<Button
					data-testid="vaultDetailDepositButton"
					color="dark"
					on:click={() => handleDepositModal(data, $vaultDetailQuery.refetch)}
					><ArrowDownOutline size="xs" class="mr-2" />Deposit</Button
				>
				<Button
					data-testid="vaultDetailWithdrawButton"
					color="dark"
					on:click={() => handleWithdrawModal(data, $vaultDetailQuery.refetch)}
					><ArrowUpOutline size="xs" class="mr-2" />Withdraw</Button
				>
			{/if}
			<Button
				data-testid="refresh-button"
				class="flex gap-1"
				on:click={() =>
					queryClient.invalidateQueries({
						queryKey: [id],
						refetchType: 'all',
						exact: false
					})}
				>Refresh <Refresh
					spin={$vaultDetailQuery.isLoading || $vaultDetailQuery.isFetching}
				/></Button
			>
		</div>
	</svelte:fragment>
	<svelte:fragment slot="card" let:data>
		<CardProperty data-testid="vaultDetailVaultId">
			<svelte:fragment slot="key">Vault ID</svelte:fragment>
			<svelte:fragment slot="value">{bigintStringToHex(data.vaultId)}</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailOrderbookAddress">
			<svelte:fragment slot="key">Orderbook</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash type={HashType.Identifier} value={data.orderbook.id} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailOwnerAddress">
			<svelte:fragment slot="key">Owner Address</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash type={HashType.Wallet} value={data.owner} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailTokenAddress">
			<svelte:fragment slot="key">Token address</svelte:fragment>
			<svelte:fragment slot="value">
				<Hash value={data.token.id} />
			</svelte:fragment>
		</CardProperty>

		<CardProperty data-testid="vaultDetailBalance">
			<svelte:fragment slot="key">Balance</svelte:fragment>
			<svelte:fragment slot="value"
				>{formatUnits(BigInt(data.balance), Number(data.token.decimals ?? 0))}
				{data.token.symbol}</svelte:fragment
			>
		</CardProperty>

		<CardProperty>
			<svelte:fragment slot="key">Orders as input</svelte:fragment>
			<svelte:fragment slot="value">
				<p data-testid="vaultDetailOrdersAsInput" class="flex flex-wrap justify-start">
					{#if data.ordersAsInput && data.ordersAsInput.length > 0}
						{#each data.ordersAsInput as order}
							<OrderOrVaultHash
								type="orders"
								orderOrVault={order}
								{network}
								{updateActiveNetworkAndOrderbook}
							/>
						{/each}
					{:else}
						None
					{/if}
				</p>
			</svelte:fragment>
		</CardProperty>

		<CardProperty>
			<svelte:fragment slot="key">Orders as output</svelte:fragment>
			<svelte:fragment slot="value">
				<p data-testid="vaulDetailOrdersAsOutput" class="flex flex-wrap justify-start">
					{#if data.ordersAsOutput && data.ordersAsOutput.length > 0}
						{#each data.ordersAsOutput as order}
							<OrderOrVaultHash
								type="orders"
								orderOrVault={order}
								{network}
								{updateActiveNetworkAndOrderbook}
							/>
						{/each}
					{:else}
						None
					{/if}
				</p>
			</svelte:fragment>
		</CardProperty>
	</svelte:fragment>

	<svelte:fragment slot="chart" let:data>
		<VaultBalanceChart vault={data} {subgraphUrl} {lightweightChartsTheme} {id} />
	</svelte:fragment>

	<svelte:fragment slot="below"><VaultBalanceChangesTable {id} {subgraphUrl} /></svelte:fragment>
</TanstackPageContentDetail>
