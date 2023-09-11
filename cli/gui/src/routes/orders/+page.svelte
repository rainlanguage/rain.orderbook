<script lang="ts">
	import { queries, shortenHexString } from '$lib';
	import { account } from 'svelte-wagmi-stores';
	import {
		Badge,
		Button,
		FloatingLabelInput,
		Indicator,
		Popover,
		Spinner,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import Property from '$lib/components/Property.svelte';
	import { toHex } from 'viem';
	import { fade } from 'svelte/transition';

	const { result, refresh, owners, orders, validInputs, validOutputs } = queries.queryOrders();
	let owner: string, order: string;
	$: $owners = owner ? [owner] : null;
	$: $orders = order ? [order] : null;
</script>

<div class="flex flex-col gap-y-2 items-start border border-gray-200 p-8 rounded-md mb-6">
	<FloatingLabelInput label="Order" style="outlined" type="text" bind:value={order} />
	<FloatingLabelInput label="Owner" style="outlined" type="text" bind:value={owner} />
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
		<TableHeadCell>Active</TableHeadCell>
		<TableHeadCell>Orderhash</TableHeadCell>
		<TableHeadCell>Added</TableHeadCell>
		<TableHeadCell>Owner</TableHeadCell>
		<TableHeadCell>Valid inputs</TableHeadCell>
		<TableHeadCell>Valid outputs</TableHeadCell>
	</TableHead>
	<TableBody tableBodyClass="divide-y font-regular">
		{#if $result?.data}
			{#each $result.data as order, i}
				<TableBodyRow>
					<TableBodyCell
						><Indicator
							class="mx-auto"
							color={order.orderActive ? 'green' : 'gray'}
						/></TableBodyCell
					>
					<TableBodyCell>{shortenHexString(order.orderHash)}</TableBodyCell>
					<TableBodyCell>{new Date(order.timestamp * 1000).toLocaleDateString()}</TableBodyCell>
					<TableBodyCell>{shortenHexString(order.owner.id)}</TableBodyCell>
					<TableBodyCell>
						{#if order.validInputs}
							<div class="flex flex-row gap-2 flex-wrap max-w-md">
								{#each order.validInputs as input, y}
									{@const triggerId = `${input.tokenVault.token.symbol}-${i}-${y}`}
									<Badge class="cursor-default" id={triggerId} color="dark" large>
										${input.tokenVault.token.symbol}
									</Badge>
									<Popover
										transition={fade}
										params={{ duration: 100 }}
										triggeredBy={`#${triggerId}`}
									>
										<Property label="Token address">
											{input.tokenVault.token.symbol}
										</Property>
										<Property label="Vault balance">
											{input.tokenVault.balanceDisplay}
										</Property>
										<Property label="Vault ID">
											{shortenHexString(input.vaultId)}
										</Property>
									</Popover>
								{/each}
							</div>
						{/if}
					</TableBodyCell>
					<TableBodyCell>
						{#if order.validOutputs}
							<div class="flex flex-row gap-2 flex-wrap max-w-md">
								{#each order.validOutputs as output, y}
									{@const triggerIdd = `${output.tokenVault.token.symbol}-${i}-${y}`}
									<Badge class="cursor-default" id={triggerIdd} color="dark" large>
										${output.tokenVault.token.symbol}
									</Badge>
									<Popover
										transition={fade}
										params={{ duration: 100 }}
										triggeredBy={`#${triggerIdd}`}
									>
										<Property label="Token address">
											{output.tokenVault.token.symbol}
										</Property>
										<Property label="Vault balance">
											{output.tokenVault.balanceDisplay}
										</Property>
										<Property label="Vault ID">
											{shortenHexString(toHex(BigInt(output.vaultId)))}
										</Property>
									</Popover>
								{/each}
							</div>
						{/if}
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
