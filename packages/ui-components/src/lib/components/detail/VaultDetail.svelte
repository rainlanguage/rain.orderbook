<script lang="ts">
	import { bigintStringToHex } from '../../utils/hex';
	import Hash, { HashType } from '../Hash.svelte';
	import VaultBalanceChangesTable from '../tables/VaultBalanceChangesTable.svelte';
	import VaultBalanceChart from '../charts/VaultBalanceChart.svelte';
	import TanstackPageContentDetail from './TanstackPageContentDetail.svelte';
	import CardProperty from '../CardProperty.svelte';
	import { QKEY_VAULT } from '../../queries/keys';
	import { getVault, type SgVault } from '@rainlanguage/orderbook';
	import type { ChartTheme } from '../../utils/lightweightChartsThemes';
	import { formatUnits, isAddress, isAddressEqual } from 'viem';
	import { createQuery } from '@tanstack/svelte-query';
	import { onDestroy } from 'svelte';
	import type { Readable } from 'svelte/store';
	import { useQueryClient } from '@tanstack/svelte-query';
	import OrderOrVaultHash from '../OrderOrVaultHash.svelte';
	import type { AppStoresInterface } from '../../types/appStores';
	import Refresh from '../icon/Refresh.svelte';
	import { invalidateTanstackQueries } from '$lib/queries/queryClient';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { Button } from 'flowbite-svelte';
	import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';

	export let id: string;
	export let network: string;
	export let lightweightChartsTheme: Readable<ChartTheme> | undefined = undefined;
	export let activeNetworkRef: AppStoresInterface['activeNetworkRef'];
	export let activeOrderbookRef: AppStoresInterface['activeOrderbookRef'];
	export let settings;

	/**
	 * Required callback function when deposit action is triggered for a vault
	 * @param vault The vault to deposit into
	 */
	export let onDeposit: (vault: SgVault) => void;

	/**
	 * Required callback function when withdraw action is triggered for a vault
	 * @param vault The vault to withdraw from
	 */
	export let onWithdraw: (vault: SgVault) => void;

	const subgraphUrl = $settings?.subgraphs?.[network] || '';
	const queryClient = useQueryClient();
	const { account } = useAccount();

	$: vaultDetailQuery = createQuery<SgVault>({
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
		invalidateTanstackQueries(queryClient, [id, QKEY_VAULT + id]);
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
		<div class="flex items-center gap-2">
			{#if $account && isAddress($account) && isAddress(data.owner) && isAddressEqual($account, data.owner)}
				<Button
					color="light"
					size="xs"
					data-testid="deposit-button"
					aria-label="Deposit to vault"
					on:click={() => onDeposit(data)}
				>
					<ArrowDownOutline size="xs" />
				</Button>
				<Button
					color="light"
					size="xs"
					data-testid="withdraw-button"
					aria-label="Withdraw from vault"
					on:click={() => onWithdraw(data)}
				>
					<ArrowUpOutline size="xs" />
				</Button>
			{/if}

			<Refresh
				on:click={() => invalidateTanstackQueries(queryClient, [id, QKEY_VAULT + id])}
				spin={$vaultDetailQuery.isLoading || $vaultDetailQuery.isFetching}
			/>
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
				<p data-testid="vaultDetailOrdersAsOutput" class="flex flex-wrap justify-start">
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
