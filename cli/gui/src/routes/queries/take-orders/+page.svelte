<script lang="ts">
	import { queries, orderbook, shortenHexString } from '$lib';
	import ClipboardCopy from '$lib/components/ClipboardCopy.svelte';
	import {
		Button,
		FloatingLabelInput,
		Spinner,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import { account } from 'svelte-wagmi-stores';
	const { result, refresh, owners, inputTokens, outputTokens, orders } =
		queries.queryTakeOrderEntities();

	let owner: string, order: string, inputToken: string, outputToken: string;
	$: $owners = owner ? [owner] : null;
	$: $orders = order ? [order] : null;
	$: $inputTokens = inputToken ? [inputToken] : null;
	$: $outputTokens = outputToken ? [outputToken] : null;
</script>

<div class="flex flex-col gap-y-2 items-start border border-gray-200 p-8 rounded-md mb-6">
	<FloatingLabelInput label="Order" style="outlined" type="text" bind:value={order} />
	<FloatingLabelInput label="Owner" style="outlined" type="text" bind:value={owner} />
	<FloatingLabelInput label="Output token" style="outlined" type="text" bind:value={inputToken} />
	<FloatingLabelInput label="Input token" style="outlined" type="text" bind:value={outputToken} />

	{#if $account?.address}
		<Button
			class="whitespace-nowrap"
			on:click={() => {
				owner = $account?.address?.toLowerCase() || '';
			}}>Show only mine</Button
		>
	{/if}
</div>
<div class="mb-6 w-full flex justify-end">
	<Button size="xs" on:click={refresh}>Refresh</Button>
</div>
<Table divClass="overflow-x-scroll" shadow>
	<TableHead>
		<TableHeadCell>Order ID</TableHeadCell>
		<TableHeadCell>Owner</TableHeadCell>
		<TableHeadCell>Transaction</TableHeadCell>
		<TableHeadCell>Time</TableHeadCell>
		<TableHeadCell>Input</TableHeadCell>
		<TableHeadCell>Output</TableHeadCell>
	</TableHead>
	<TableBody tableBodyClass="divide-y font-regular">
		{#if $result?.data}
			{#each $result.data as takeOrder}
				<TableBodyRow>
					<TableBodyCell>{shortenHexString(takeOrder.order.orderHash)}</TableBodyCell>
					<TableBodyCell>
						<ClipboardCopy copyContent={takeOrder.order.owner.id}>
							{shortenHexString(takeOrder.order.owner.id)}
						</ClipboardCopy>
					</TableBodyCell>
					<TableBodyCell
						><ClipboardCopy copyContent={takeOrder.transaction.id}
							>{shortenHexString(takeOrder.transaction.id)}</ClipboardCopy
						></TableBodyCell
					>
					<TableBodyCell>{new Date(takeOrder.timestamp * 1000).toUTCString()}</TableBodyCell>
					<TableBodyCell>
						<p>
							{takeOrder.inputDisplay}
							{takeOrder.inputToken.symbol}
						</p>
						<p class="text-gray-400">
							{takeOrder.outputToken.id}
						</p>
					</TableBodyCell>
					<TableBodyCell>
						<p>
							{takeOrder.outputDisplay}
							{takeOrder.outputToken.symbol}
						</p>
						<p class="text-gray-400">
							{takeOrder.inputToken.id}
						</p>
					</TableBodyCell>
				</TableBodyRow>
			{/each}
		{:else if $result?.error}
			{JSON.stringify($result.error)}
		{:else}
			<div class="w-full flex justify-center items-center py-4">
				<Spinner />
			</div>
		{/if}
	</TableBody>
</Table>
