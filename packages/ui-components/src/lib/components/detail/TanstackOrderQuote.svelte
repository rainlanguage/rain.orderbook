<script lang="ts">
	import { useToasts } from '$lib/providers/toasts/useToasts';
	import { invalidateTanstackQueries } from '$lib/queries/queryClient';
	import Refresh from '../icon/Refresh.svelte';
	import EditableSpan from '../EditableSpan.svelte';
	import { QKEY_ORDER_QUOTE } from '../../queries/keys';
	import { hexToNumber, isHex } from 'viem';
	import { createQuery, useQueryClient } from '@tanstack/svelte-query';
	import type { RaindexOrder, RaindexOrderQuote } from '@rainlanguage/orderbook';
	import {
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import { BugOutline, PauseSolid, PlaySolid } from 'flowbite-svelte-icons';
	import Tooltip from '../Tooltip.svelte';

	export let order: RaindexOrder;
	export let handleQuoteDebugModal:
		| undefined
		| ((
				order: RaindexOrder,
				inputIndex: number,
				outputIndex: number,
				pairName: string,
				blockNumber?: bigint
		  ) => void) = undefined;

	let enabled = true;

	const queryClient = useQueryClient();
	const { errToast } = useToasts();

	const refreshQuotes = async () => {
		try {
			await invalidateTanstackQueries(queryClient, [order.id, QKEY_ORDER_QUOTE + order.id]);
		} catch {
			errToast('Failed to refresh');
		}
	};

	$: orderQuoteQuery = createQuery<RaindexOrderQuote[]>({
		queryKey: [order.id, QKEY_ORDER_QUOTE + order.id],
		queryFn: async () => {
			const result = await order.getQuotes(blockNumber);
			if (result.error) throw new Error(result.error.msg);
			return result.value;
		},
		enabled: !!order.id && enabled
	});

	let blockNumber: bigint | undefined;
	$: orderModalArg = order;
</script>

<div class="mt-4">
	<div class="mb-4 flex items-center justify-between">
		<h2 class="text-lg font-semibold">Order quotes</h2>
		<div class="flex items-center gap-x-1">
			{#if $orderQuoteQuery.data && $orderQuoteQuery.data.length > 0 && isHex($orderQuoteQuery.data[0].blockNumber)}
				<EditableSpan
					displayValue={blockNumber?.toString() ||
						hexToNumber($orderQuoteQuery.data[0].blockNumber).toString()}
					on:focus={() => {
						enabled = false;
					}}
					on:blur={({ detail: { value } }) => {
						blockNumber = BigInt(value);
						refreshQuotes();
					}}
				/>
			{/if}
			<span></span>
			<Refresh
				class="h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
				on:click={refreshQuotes}
				spin={$orderQuoteQuery.isLoading || $orderQuoteQuery.isFetching}
			/>
			<PauseSolid
				class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${!enabled ? 'hidden' : ''}`}
				on:click={() => {
					enabled = false;
				}}
			/>
			<PlaySolid
				on:click={() => {
					enabled = true;
					blockNumber = undefined;
					refreshQuotes();
				}}
				class={`ml-2 h-8 w-3 cursor-pointer text-gray-400 dark:text-gray-400 ${enabled ? 'hidden' : ''}`}
			/>
		</div>
	</div>

	<Table divClass="rounded-lg lg:overflow-hidden overflow-auto dark:border-none border">
		<TableHead data-testid="head">
			<TableHeadCell class="w-[80px]" data-testid="orderQuotesPair">Pair</TableHeadCell>
			<TableHeadCell class="w-1/4" data-testid="orderQuotesMaxOutput">Maximum Output</TableHeadCell>
			<TableHeadCell class="w-1/4" data-testid="orderQuotesPrice">Price</TableHeadCell>
			<TableHeadCell data-testid="orderQuotesPrice">Maximum Input</TableHeadCell>
			<TableHeadCell class="w-[50px]" />
		</TableHead>

		<TableBody>
			{#if $orderQuoteQuery.data && $orderQuoteQuery.data.length > 0}
				{#each $orderQuoteQuery.data as item, index}
					{#if item.success && item.data}
						<TableBodyRow data-testid="bodyRow">
							<TableBodyCell>{item.pair.pairName}</TableBodyCell>
							<TableBodyCell>{item.data.formattedMaxOutput}</TableBodyCell>
							<TableBodyCell
								>{item.data.formattedRatio}
								<span class="text-gray-400">({item.data.formattedInverseRatio})</span
								></TableBodyCell
							>
							<TableBodyCell>{item.data.formattedMaxInput}</TableBodyCell>
							<TableBodyCell>
								{#if handleQuoteDebugModal}
									<button
										on:click={() =>
											handleQuoteDebugModal(
												orderModalArg,
												item.pair.inputIndex,
												item.pair.outputIndex,
												item.pair.pairName,
												BigInt($orderQuoteQuery.data[0].blockNumber)
											)}
									>
										<BugOutline size="sm" color="grey" />
									</button>
								{/if}
							</TableBodyCell>
						</TableBodyRow>
					{:else if !item.success && item.error}
						<TableBodyRow>
							<TableBodyCell>{item.pair.pairName}</TableBodyCell>
							<TableBodyCell colspan="3" class="text-sm text-red-500 dark:text-red-400">
								<Tooltip
									triggeredBy={`#quote-error-${index}`}
									customClass="max-w-sm whitespace-pre-wrap break-words"
								>
									{item.error}
								</Tooltip>
								<div
									id={`quote-error-${index}`}
									class="max-w-xl truncate cursor-pointer self-start border-dotted border-red-500 pr-2"
								>
									{item.error}
								</div>
							</TableBodyCell>
							<TableBodyCell>
								{#if handleQuoteDebugModal}
									<button
										on:click={() =>
											handleQuoteDebugModal(
												order,
												item.pair.inputIndex,
												item.pair.outputIndex,
												item.pair.pairName,
												BigInt($orderQuoteQuery.data[0].blockNumber)
											)}
									>
										<BugOutline size="sm" color="grey" />
									</button>
								{/if}
							</TableBodyCell>
						</TableBodyRow>
					{/if}
				{/each}
			{:else if $orderQuoteQuery.isError}
				<TableBodyRow>
					<TableBodyCell colspan="3" class="text-center text-red-500 dark:text-red-400">
						{'Error fetching quotes:'} <br />
						{$orderQuoteQuery.error}
					</TableBodyCell>
				</TableBodyRow>
			{/if}
		</TableBody>
	</Table>
</div>
