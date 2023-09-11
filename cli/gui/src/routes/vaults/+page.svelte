<script lang="ts">
	import { shortenHexString } from '$lib';
	import { queries } from '$lib';
	import { account } from 'svelte-wagmi-stores';
	import { toHex } from 'viem';
	import {
		Badge,
		Button,
		FloatingLabelInput,
		Popover,
		Spinner,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import { goto } from '$app/navigation';
	import { fade } from 'svelte/transition';
	import { Icon } from 'flowbite-svelte-icons';

	const { result, refresh, owners, orders, tokens } = queries.queryTokenVaults();

	let owner: string, order: string, token: string;
	$: $owners = owner ? [owner] : null;
	$: $orders = order ? [order] : null;
	$: $tokens = token ? [token] : null;
</script>

<div class="flex flex-col gap-y-2 items-start border border-gray-200 p-8 rounded-md mb-6">
	<FloatingLabelInput label="Order" style="outlined" type="text" bind:value={order} />
	<FloatingLabelInput label="Owner" style="outlined" type="text" bind:value={owner} />
	<FloatingLabelInput label="Token" style="outlined" type="text" bind:value={token} />
	{#if $account?.address}
		<Button
			class="whitespace-nowrap"
			on:click={() => {
				owner = $account?.address?.toLowerCase() || '';
			}}>Show only mine</Button
		>
	{/if}
</div>
<div class="mb-6 w-full flex justify-end gap-x-2">
	<Button
		outline
		on:click={() => {
			goto('/vaults/new');
		}}><Icon name="circle-plus-solid" class="mr-2 w-4" />New vault</Button
	>
	<Button size="xs" on:click={refresh}>Refresh</Button>
</div>
<Table divClass="overflow-x-scroll" shadow hoverable>
	<TableHead>
		<TableHeadCell>ID</TableHeadCell>
		<TableHeadCell>Owner</TableHeadCell>
		<TableHeadCell>Token</TableHeadCell>
		<TableHeadCell>Balance</TableHeadCell>
		<TableHeadCell>Linked orders</TableHeadCell>
	</TableHead>
	<TableBody tableBodyClass="divide-y font-regular">
		{#if $result?.data}
			{#each $result.data as vault, i}
				<TableBodyRow
					on:click={() => {
						goto(`/vaults/${toHex(BigInt(vault.vaultId))}/${vault.token.id}`);
					}}
				>
					<a href={`/vaults/${toHex(BigInt(vault.vaultId))}/${vault.token.id}`} class="hidden" />
					<TableBodyCell>{shortenHexString(toHex(BigInt(vault.vaultId)))}</TableBodyCell>
					<TableBodyCell>{shortenHexString(vault.owner.id)}</TableBodyCell>
					<TableBodyCell>
						{@const triggerId = `${vault.token.symbol}${i}`}
						<Badge class="cursor-default" id={triggerId} large color="dark">
							${vault.token.symbol.toUpperCase()}
						</Badge>
						<Popover transition={fade} params={{ duration: 100 }} triggeredBy={`#${triggerId}`}>
							<p>{vault.token.name}</p>
							<p class="text-gray-400">{vault.token.id}</p>
						</Popover>
					</TableBodyCell>
					<TableBodyCell>{vault.balanceDisplay}</TableBodyCell>
					<TableBodyCell>
						{#if vault.orders}
							{#each vault.orders as order}
								<Badge class="cursor-default" large color="dark">
									{shortenHexString(order.orderHash)}
								</Badge>
							{/each}
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
