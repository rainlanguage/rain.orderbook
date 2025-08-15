<script lang="ts">
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { QKEY_ORDER_TRADES_LIST } from '../../queries/keys';
	import { DEFAULT_PAGE_SIZE } from '../../queries/constants';
	import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import { formatTimestampSecondsAsLocal } from '../../services/time';
	import Hash, { HashType } from '../Hash.svelte';
	import { BugOutline } from 'flowbite-svelte-icons';
	import type { RaindexOrder, RaindexTrade } from '@rainlanguage/orderbook';
	import TableTimeFilters from '../charts/TableTimeFilters.svelte';

	export let order: RaindexOrder;
	export let rpcs: string[] | undefined = undefined;
	export let handleDebugTradeModal: ((hash: string, rpcs: string[]) => void) | undefined =
		undefined;

	let startTimestamp: number | undefined;
	let endTimestamp: number | undefined;
	let tradesCount: number | undefined;

	$: orderTradesQuery = createInfiniteQuery({
		queryKey: [order.id, QKEY_ORDER_TRADES_LIST + order.id],
		queryFn: async ({ pageParam }: { pageParam: number }) => {
			tradesCount = undefined;

			const [countResult, tradesResult] = await Promise.all([
				order.getTradeCount(
					startTimestamp ? BigInt(startTimestamp) : undefined,
					endTimestamp ? BigInt(endTimestamp) : undefined
				),
				order.getTradesList(
					startTimestamp ? BigInt(startTimestamp) : undefined,
					endTimestamp ? BigInt(endTimestamp) : undefined,
					pageParam + 1
				)
			]);
			if (countResult.error) throw new Error(countResult.error.readableMsg);
			if (tradesResult.error) throw new Error(tradesResult.error.readableMsg);

			const count = countResult.value;
			const trades = tradesResult.value;

			if (typeof count === 'number') {
				tradesCount = count;
			}

			return trades;
		},
		initialPageParam: 0,
		getNextPageParam: (
			lastPage: RaindexTrade[],
			_allPages: RaindexTrade[][],
			lastPageParam: number
		) => {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		}
	});

	const AppTable = TanstackAppTable<RaindexTrade>;
</script>

<AppTable
	query={orderTradesQuery}
	emptyMessage="No trades found"
	rowHoverable={false}
	queryKey={order.id}
>
	<svelte:fragment slot="info">
		{#if tradesCount !== undefined}
			<div class="px-2">{tradesCount} Trades</div>
		{/if}
	</svelte:fragment>
	<svelte:fragment slot="timeFilter">
		<TableTimeFilters bind:startTimestamp bind:endTimestamp />
	</svelte:fragment>
	<svelte:fragment slot="head">
		<TableHeadCell padding="p-4">Date</TableHeadCell>
		<TableHeadCell padding="p-0">Sender</TableHeadCell>
		<TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
		<TableHeadCell padding="p-0">Input</TableHeadCell>
		<TableHeadCell padding="p-0">Output</TableHeadCell>
		<TableHeadCell padding="p-0">IO Ratio</TableHeadCell>
		<TableHeadCell padding="p-0"></TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell tdClass="px-4 py-2">
			{formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-32">
			<Hash type={HashType.Wallet} value={item.transaction.from} />
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-32">
			<Hash type={HashType.Transaction} value={item.transaction.id} />
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2">
			{item.inputVaultBalanceChange.formattedAmount}
			{item.inputVaultBalanceChange.token.symbol}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2">
			{item.outputVaultBalanceChange.formattedAmount}
			{item.outputVaultBalanceChange.token.symbol}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2" data-testid="io-ratio">
			{Math.abs(
				Number(item.inputVaultBalanceChange.formattedAmount) /
					Number(item.outputVaultBalanceChange.formattedAmount)
			)}
			<span class="text-gray-400">
				({Math.abs(
					Number(item.outputVaultBalanceChange.formattedAmount) /
						Number(item.inputVaultBalanceChange.formattedAmount)
				)})
			</span>
		</TableBodyCell>
		{#if rpcs && handleDebugTradeModal}
			<TableBodyCell tdClass="py-2">
				<button
					data-testid="debug-trade-button"
					class="text-gray-500 hover:text-gray-700"
					on:click={() => {
						if (rpcs) handleDebugTradeModal(item.transaction.id, rpcs);
					}}
				>
					<BugOutline size="xs" />
				</button>
			</TableBodyCell>
		{/if}
	</svelte:fragment>
</AppTable>
