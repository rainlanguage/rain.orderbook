<script lang="ts">
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { QKEY_VAULTS_VOL_LIST } from '../../queries/keys';
	import { type RaindexOrder, type VaultVolume } from '@rainlanguage/orderbook';
	import { TableBodyCell, TableHeadCell } from 'flowbite-svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import { formatUnits } from 'viem';
	import TableTimeFilters from '../charts/TableTimeFilters.svelte';
	import { bigintStringToHex } from '../../utils/hex';

	export let order: RaindexOrder;

	let startTimestamp: number | undefined;
	let endTimestamp: number | undefined;

	$: queryStartTime = startTimestamp ? BigInt(startTimestamp) : undefined;
	$: queryEndTime = endTimestamp ? BigInt(endTimestamp) : undefined;

	$: vaultsVol = createInfiniteQuery<VaultVolume[]>({
		queryKey: [order.id, QKEY_VAULTS_VOL_LIST + order.id],
		queryFn: async () => {
			const result = await order.getVaultsVolume(queryStartTime, queryEndTime);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		},
		initialPageParam: 0,
		getNextPageParam: () => undefined
	});
</script>

<TanstackAppTable
	query={vaultsVol}
	emptyMessage="No trades found"
	rowHoverable={false}
	queryKey={order.id}
>
	<svelte:fragment slot="timeFilter">
		<TableTimeFilters bind:startTimestamp bind:endTimestamp />
	</svelte:fragment>
	<svelte:fragment slot="head">
		<TableHeadCell padding="p-4">Vault</TableHeadCell>
		<TableHeadCell padding="p-0">Token</TableHeadCell>
		<TableHeadCell padding="p-0">In Volume</TableHeadCell>
		<TableHeadCell padding="p-0">Out Volume</TableHeadCell>
		<TableHeadCell padding="p-0">Net Volume</TableHeadCell>
		<TableHeadCell padding="p-0">Total Volume</TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell tdClass="px-4 py-2">
			<Hash type={HashType.Identifier} shorten value={bigintStringToHex(item.id)} />
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-32">
			<div class="flex gap-x-3">
				<Hash type={HashType.Address} shorten value={item.token.address} />
				{item.token.symbol}
			</div>
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-32" data-testid="total-in">
			{formatUnits(BigInt(item.volDetails.totalIn), Number(item.token.decimals ?? 0))}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2" data-testid="total-out">
			{formatUnits(BigInt(item.volDetails.totalOut), Number(item.token.decimals ?? 0))}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2" data-testid="net-vol">
			{(BigInt(item.volDetails.totalIn) >= BigInt(item.volDetails.totalOut) ? '' : '-') +
				formatUnits(BigInt(item.volDetails.netVol), Number(item.token.decimals ?? 0))}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2" data-testid="total-vol">
			{formatUnits(BigInt(item.volDetails.totalVol), Number(item.token.decimals ?? 0))}
		</TableBodyCell>
	</svelte:fragment>
</TanstackAppTable>
