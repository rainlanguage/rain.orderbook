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
	import { queryClient } from '../../stores/queryClient';
	import { getOrder, type Order } from '@rainlanguage/orderbook/js_api';
	import { createQuery } from '@tanstack/svelte-query';
	import { Button, TabItem, Tabs } from 'flowbite-svelte';
	import { onDestroy } from 'svelte';
	import type { Readable } from 'svelte/store';
	import OrderApy from '../tables/OrderAPY.svelte';
	import { page } from '$app/stores';

	export let walletAddressMatchesOrBlank: Readable<(address: string) => boolean> | undefined =
		undefined;
	export let handleOrderRemoveModal: ((order: Order, refetch: () => void) => void) | undefined =
		undefined;
	export let handleQuoteDebugModal:
		| undefined
		| ((
				order: Order,
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
	let codeMirrorDisabled = true;
	let codeMirrorStyles = {};

	$: orderDetailQuery = createQuery<Order>({
		queryKey: [id, QKEY_ORDER + id],
		queryFn: () => getOrder(subgraphUrl, id),
		enabled: !!subgraphUrl
	});

	const interval = setInterval(async () => {
		// This invalidate function invalidates
		// both order detail and order trades list queries
		await queryClient.invalidateQueries({
			queryKey: [id],
			refetchType: 'active',
			exact: false
		});
	}, 10000);

	onDestroy(() => {
		clearInterval(interval);
	});

	$: subgraphName = $page.url.pathname.split('/')[2]?.split('-')[0];
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
	<svelte:fragment slot="top" let:data>
		<div class="flex gap-x-4 text-3xl font-medium dark:text-white">
			<div class="flex gap-x-2">
				<span class="font-light">Order</span>
				<Hash shorten value={data.orderHash} />
			</div>
			<BadgeActive active={data.active} large />
		</div>
		{#if data && $walletAddressMatchesOrBlank?.(data.owner) && data.active && handleOrderRemoveModal}
			<Button
				data-testid="remove-button"
				color="dark"
				on:click={() => handleOrderRemoveModal(data, $orderDetailQuery.refetch)}
				disabled={!handleOrderRemoveModal}
			>
				Remove
			</Button>
		{/if}
	</svelte:fragment>
	<svelte:fragment slot="card" let:data>
		<div class="flex flex-col gap-y-6">
			<CardProperty>
				<svelte:fragment slot="key">Orderbook</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Identifier} shorten={false} value={data.orderbook.id} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Owner</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Wallet} shorten={false} value={data.owner} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Created</svelte:fragment>
				<svelte:fragment slot="value">
					{formatTimestampSecondsAsLocal(BigInt(data.timestampAdded))}
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Input vaults</svelte:fragment>
				<svelte:fragment slot="value">
					<div class="mb-2 flex justify-end">
						<span>Balance</span>
					</div>
					<div class="space-y-2">
						{#each data.inputs || [] as t}
							<ButtonVaultLink tokenVault={t} {subgraphName} />
						{/each}
					</div>
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Output vaults</svelte:fragment>
				<svelte:fragment slot="value">
					<div class="mb-2 flex justify-end">
						<span>Balance</span>
					</div>
					<div class="space-y-2">
						{#each data.outputs || [] as t}
							<ButtonVaultLink tokenVault={t} {subgraphName} />
						{/each}
					</div>
				</svelte:fragment>
			</CardProperty>
		</div>
	</svelte:fragment>
	<svelte:fragment slot="chart">
		<OrderTradesChart {id} {subgraphUrl} {lightweightChartsTheme} {colorTheme} />
	</svelte:fragment>
	<svelte:fragment slot="below" let:data>
		<TanstackOrderQuote {id} order={data} {rpcUrl} {orderbookAddress} {handleQuoteDebugModal} />
		<Tabs
			style="underline"
			contentClass="mt-4"
			defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
		>
			<TabItem open title="Rainlang source">
				<div class="mb-8 overflow-hidden rounded-lg border dark:border-none">
					<CodeMirrorRainlang
						order={data}
						codeMirrorTheme={$codeMirrorTheme}
						{codeMirrorDisabled}
						{codeMirrorStyles}
					></CodeMirrorRainlang>
				</div>
			</TabItem>
			<TabItem title="Trades">
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
