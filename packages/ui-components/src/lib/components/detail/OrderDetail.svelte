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

	import { getOrder, type Order } from '@rainlanguage/orderbook/js_api';
	import { createQuery } from '@tanstack/svelte-query';
	import { Button, TabItem, Tabs } from 'flowbite-svelte';

	export let walletAddressMatchesOrBlank: ((address: string) => boolean) | undefined = undefined;
	export let handleOrderRemoveModal: ((order: Order, refetch: () => void) => void) | undefined =
		undefined;
	export let handleQuoteDebugModal:
		| ((order: Order, rpcUrl: string, orderbookAddress: string) => void)
		| undefined = undefined;
	export let handleDebugTradeModal: ((hash: string, rpcUrl: string) => void) | undefined =
		undefined;

	export let colorTheme;
	export let codeMirrorTheme;
	export let lightweightChartsTheme;
	export let orderbookAddress: string | undefined = undefined;
	export let id: string;
	export let rpcUrl: string;
	export let subgraphUrl: string;

	$: orderDetailQuery = createQuery<Order>({
		queryKey: [id, QKEY_ORDER + id],
		queryFn: () => getOrder(subgraphUrl, id || ''),
		enabled: !!subgraphUrl && !!id
	});
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
	<svelte:fragment slot="top" let:data={order}>
		<div class="flex gap-x-4 text-3xl font-medium dark:text-white">
			<div class="flex gap-x-2">
				<span class="font-light">Order</span>
				<Hash shorten value={order.orderHash} />
			</div>
			<BadgeActive active={order.active} large />
		</div>
		{#if order && walletAddressMatchesOrBlank?.(order.owner) && order.active && handleOrderRemoveModal}
			<Button
				color="dark"
				on:click={() => handleOrderRemoveModal(order, $orderDetailQuery.refetch)}
				disabled={!handleOrderRemoveModal}
			>
				Remove
			</Button>
		{/if}
	</svelte:fragment>
	<svelte:fragment slot="card" let:data={order}>
		<div class="flex flex-col gap-y-6">
			<CardProperty>
				<svelte:fragment slot="key">Orderbook</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Identifier} shorten={false} value={order.orderbook.id} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Owner</svelte:fragment>
				<svelte:fragment slot="value">
					<Hash type={HashType.Wallet} shorten={false} value={order.owner} />
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Created</svelte:fragment>
				<svelte:fragment slot="value">
					{formatTimestampSecondsAsLocal(BigInt(order.timestampAdded))}
				</svelte:fragment>
			</CardProperty>

			<CardProperty>
				<svelte:fragment slot="key">Input vaults</svelte:fragment>
				<svelte:fragment slot="value">
					<div class="mb-2 flex justify-end">
						<span>Balance</span>
					</div>
					<div class="space-y-2">
						{#each order.inputs || [] as t}
							<ButtonVaultLink tokenVault={t} />
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
						{#each order.outputs || [] as t}
							<ButtonVaultLink tokenVault={t} />
						{/each}
					</div>
				</svelte:fragment>
			</CardProperty>
		</div>
	</svelte:fragment>
	<svelte:fragment slot="chart">
		<OrderTradesChart {id} {subgraphUrl} {colorTheme} {lightweightChartsTheme} />
	</svelte:fragment>
	<svelte:fragment slot="below" let:data={order}>
		<TanstackOrderQuote
			{id}
			{order}
			{rpcUrl}
			orderbookAddress={orderbookAddress || ''}
			{handleQuoteDebugModal}
		/>
		<Tabs
			style="underline"
			contentClass="mt-4"
			defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
		>
			<TabItem open title="Rainlang source">
				<div class="mb-8 overflow-hidden rounded-lg border dark:border-none">
					<CodeMirrorRainlang disabled={true} {order} {codeMirrorTheme} />
				</div>
			</TabItem>
			<TabItem title="Trades">
				<OrderTradesListTable {id} {subgraphUrl} {rpcUrl} {handleDebugTradeModal} />
			</TabItem>
			<TabItem title="Volume">
				<OrderVaultsVolTable {id} {subgraphUrl} />
			</TabItem>
		</Tabs>
	</svelte:fragment>
</TanstackPageContentDetail>
