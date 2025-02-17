<script lang="ts">
	import Hash, { HashType } from '../Hash.svelte';
	import BadgeActive from '../BadgeActive.svelte';
	import OrderTradesChart from '../charts/OrderTradesChart.svelte';
	import OrderTradesListTable from '../tables/OrderTradesListTable.svelte';
	import TanstackOrderQuote from './TanstackOrderQuote.svelte';
	import TanstackPageContentDetail from './TanstackPageContentDetail.svelte';
	import CardProperty from '../CardProperty.svelte';
	import { formatTimestampSecondsAsLocal } from '../../utils/time';
	import ButtonVaultLink from '../ButtonVaultLink.svelte';
	import OrderVaultsVolTable from '../tables/OrderVaultsVolTable.svelte';
	import { QKEY_ORDER } from '../../queries/keys';
	import CodeMirrorRainlang from '../CodeMirrorRainlang.svelte';
	import {
		getOrder,
		type OrderSubgraph,
		type OrderWithSortedVaults,
		type Vault
	} from '@rainlanguage/orderbook/js_api';
	import { createQuery } from '@tanstack/svelte-query';
	import { Button, TabItem, Tabs } from 'flowbite-svelte';
	import { onDestroy } from 'svelte';
	import type { Writable } from 'svelte/store';
	import OrderApy from '../tables/OrderAPY.svelte';
	import { page } from '$app/stores';
	import DepositOrWithdrawButtons from './DepositOrWithdrawButtons.svelte';
	import type { Config } from 'wagmi';

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
	export let handleOrderRemoveModal:
		| ((args: {
				order: OrderSubgraph;
				onRemove: () => void;
				wagmiConfig: Config;
				chainId: number;
				orderbookAddress: string;
		  }) => void)
		| undefined = undefined;
	export let handleQuoteDebugModal:
		| undefined
		| ((
				order: OrderSubgraph,
				rpcUrl: string,
				orderbook: string,
				inputIOIndex: number,
				outputIOIndex: number,
				pair: string,
				blockNumber?: number
		  ) => void) = undefined;
	export const handleDebugTradeModal: ((hash: string, rpcUrl: string) => void) | undefined =
		undefined;
	export let colorTheme;
	export let codeMirrorTheme;
	export let lightweightChartsTheme;
	export let orderbookAddress: string | undefined = undefined;
	export let id: string;
	export let rpcUrl: string;
	export let subgraphUrl: string;
	export let chainId: number | undefined;
	export let wagmiConfig: Writable<Config> | undefined = undefined;
	export let signerAddress: Writable<string | null> | undefined = undefined;
	let codeMirrorDisabled = true;
	let codeMirrorStyles = {};
	import { useQueryClient } from '@tanstack/svelte-query';
	import Refresh from '../icon/Refresh.svelte';

	const queryClient = useQueryClient();

	$: orderDetailQuery = createQuery<OrderWithSortedVaults>({
		queryKey: [id, QKEY_ORDER + id],
		queryFn: () => {
			return getOrder(subgraphUrl, id);
		},
		enabled: !!subgraphUrl
	});

	const interval = setInterval(async () => {
		await queryClient.invalidateQueries({
			queryKey: [id],
			refetchType: 'all',
			exact: false
		});
	}, 5000);

	onDestroy(() => {
		clearInterval(interval);
	});

	$: subgraphName = $page.url.pathname.split('/')[2]?.split('-')[0];
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
	<svelte:fragment slot="top" let:data>
		<div
			class="flex w-full flex-wrap items-center justify-between gap-4 text-3xl font-medium lg:justify-between dark:text-white"
		>
			<div class="flex items-center gap-x-2">
				<div class="flex gap-x-2">
					<span class="font-light">Order</span>
					<Hash shorten value={data.order.orderHash} />
				</div>

				<BadgeActive active={data.order.active} large />
			</div>

			<div class="flex items-center gap-2">
				{#if data && $signerAddress === data.order.owner && data.order.active && handleOrderRemoveModal && $wagmiConfig && chainId && orderbookAddress}
					<Button
						data-testid="remove-button"
						color="dark"
						on:click={() =>
							handleOrderRemoveModal({
								order: data.order,
								onRemove: $orderDetailQuery.refetch,
								wagmiConfig: $wagmiConfig,
								chainId,
								orderbookAddress
							})}
						disabled={!handleOrderRemoveModal}
					>
						Remove
					</Button>
				{/if}
				<Refresh
					on:click={() =>
						queryClient.invalidateQueries({
							queryKey: [id],
							refetchType: 'all',
							exact: false
						})}
					spin={$orderDetailQuery.isLoading || $orderDetailQuery.isFetching}
				/>
			</div>
		</div>
	</svelte:fragment>
	<svelte:fragment slot="card" let:data>
		<div class="flex flex-col gap-y-6">
			<CardProperty>
				<svelte:fragment slot="key">Orderbook</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Identifier} shorten={false} value={data.order.orderbook.id} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Owner</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Wallet} shorten={false} value={data.order.owner} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Created</svelte:fragment>
				<svelte:fragment slot="value">
					{formatTimestampSecondsAsLocal(BigInt(data.order.timestampAdded))}
				</svelte:fragment>
			</CardProperty>

			{#each [{ key: 'Input vaults', type: 'inputs' }, { key: 'Output vaults', type: 'outputs' }, { key: 'Input & output vaults', type: 'inputs_outputs' }] as { key, type }}
				{#if data.vaults.get(type)?.length !== 0}
					<CardProperty>
						<svelte:fragment slot="key">{key}</svelte:fragment>
						<svelte:fragment slot="value">
							<div class="mt-2 space-y-2">
								{#each data.vaults.get(type) || [] as vault}
									<ButtonVaultLink tokenVault={vault} {subgraphName}>
										<svelte:fragment slot="buttons">
											{#if handleDepositOrWithdrawModal && $signerAddress === vault.owner && chainId}
												<DepositOrWithdrawButtons
													{vault}
													{chainId}
													{rpcUrl}
													query={orderDetailQuery}
													{handleDepositOrWithdrawModal}
													{subgraphUrl}
												/>
											{/if}
										</svelte:fragment>
									</ButtonVaultLink>
								{/each}
							</div>
						</svelte:fragment>
					</CardProperty>
				{/if}
			{/each}
		</div>
	</svelte:fragment>
	<svelte:fragment slot="chart">
		<OrderTradesChart {id} {subgraphUrl} {lightweightChartsTheme} {colorTheme} />
	</svelte:fragment>
	<svelte:fragment slot="below" let:data>
		<TanstackOrderQuote
			{id}
			order={data.order}
			{rpcUrl}
			{orderbookAddress}
			{handleQuoteDebugModal}
		/>
		<Tabs
			style="underline"
			contentClass="mt-4"
			defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4 list-none"
		>
			<TabItem title="Rainlang source">
				<div class="mb-8 overflow-hidden rounded-lg border dark:border-none">
					<CodeMirrorRainlang
						order={data.order}
						codeMirrorTheme={$codeMirrorTheme}
						{codeMirrorDisabled}
						{codeMirrorStyles}
					></CodeMirrorRainlang>
				</div>
			</TabItem>
			<TabItem open title="Trades">
				<OrderTradesListTable {id} {subgraphUrl} />
			</TabItem>
			<TabItem title="Volume">
				<OrderVaultsVolTable {id} {subgraphUrl} />
			</TabItem>
			<TabItem title="APY">
				<OrderApy {id} {subgraphUrl} />
			</TabItem>
		</Tabs>
	</svelte:fragment>
</TanstackPageContentDetail>
