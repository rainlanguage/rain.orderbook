<script lang="ts" generics="T">
	import { Heading, TableHeadCell, TableBodyCell } from 'flowbite-svelte';
	import { formatUnits } from 'viem';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import { RaindexVault, type RaindexVaultBalanceChange } from '@rainlanguage/orderbook';
	import { formatTimestampSecondsAsLocal } from '../../services/time';
	import Hash, { HashType } from '../Hash.svelte';
	import { QKEY_VAULT_CHANGES } from '../../queries/keys';
	import { DEFAULT_PAGE_SIZE } from '../../queries/constants';
	import TanstackAppTable from '../TanstackAppTable.svelte';

	export let vault: RaindexVault;

	$: balanceChangesQuery = createInfiniteQuery({
		queryKey: [vault.id, QKEY_VAULT_CHANGES + vault.id],
		queryFn: async ({ pageParam }) => {
			const result = await vault.getBalanceChanges(pageParam + 1);
			if (result.error) throw new Error(result.error.msg);
			return result.value;
		},
		initialPageParam: 0,
		getNextPageParam(lastPage, _allPages, lastPageParam) {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		}
	});

	const AppTable = TanstackAppTable<RaindexVaultBalanceChange>;
</script>

<AppTable
	query={balanceChangesQuery}
	queryKey={vault.id}
	emptyMessage="No deposits or withdrawals found"
	rowHoverable={false}
>
	<svelte:fragment slot="title">
		<Heading tag="h5" class="mb-4 mt-6 font-normal">Vault balance changes</Heading>
	</svelte:fragment>
	<svelte:fragment slot="head">
		<TableHeadCell padding="p-4">Date</TableHeadCell>
		<TableHeadCell padding="p-0">Sender</TableHeadCell>
		<TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
		<TableHeadCell padding="p-0">Balance Change</TableHeadCell>
		<TableHeadCell padding="p-0">Balance</TableHeadCell>
		<TableHeadCell padding="p--">Type</TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell tdClass="px-4 py-2" data-testid="vaultBalanceChangesTableDate">
			{formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-48" data-testid="vaultBalanceChangesTableFrom">
			<Hash type={HashType.Wallet} value={item.transaction.from} />
		</TableBodyCell>
		<TableBodyCell tdClass="break-all py-2 min-w-48" data-testid="vaultBalanceChangesTableTx">
			<Hash type={HashType.Transaction} value={item.transaction.id} />
		</TableBodyCell>
		<TableBodyCell
			tdClass="break-word p-0 text-left"
			data-testid="vaultBalanceChangesTableBalanceChange"
		>
			{formatUnits(BigInt(item.amount), Number(item.token.decimals ?? 0))}
			{item.token.symbol}
		</TableBodyCell>
		<TableBodyCell tdClass="break-word p-0 text-left" data-testid="vaultBalanceChangesTableBalance">
			{formatUnits(item.newBalance, Number(item.token.decimals ?? 0))}
			{item.token.symbol}
		</TableBodyCell>
		<TableBodyCell tdClass="break-word p-0 text-left" data-testid="vaultBalanceChangesTableType">
			{item.type}
		</TableBodyCell>
	</svelte:fragment>
</AppTable>
