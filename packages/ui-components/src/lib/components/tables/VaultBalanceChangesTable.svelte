<script lang="ts" generics="T">
	import { Heading, TableHeadCell, TableBodyCell } from 'flowbite-svelte';
	import { formatUnits } from 'viem';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import {
		getVaultBalanceChanges,
		type SgClearBounty,
		type SgDeposit,
		type SgTradeVaultBalanceChange,
		type SgWithdrawal
	} from '@rainlanguage/orderbook/js_api';
	import { formatTimestampSecondsAsLocal } from '../../utils/time';
	import Hash, { HashType } from '../Hash.svelte';
	import { QKEY_VAULT_CHANGES } from '../../queries/keys';
	import { DEFAULT_PAGE_SIZE } from '../../queries/constants';
	import TanstackAppTable from '../TanstackAppTable.svelte';

	export let id: string;
	export let subgraphUrl: string;

	$: balanceChangesQuery = createInfiniteQuery({
		queryKey: [id, QKEY_VAULT_CHANGES + id],
		queryFn: ({ pageParam }) => {
			return getVaultBalanceChanges(subgraphUrl || '', id, {
				page: pageParam + 1,
				pageSize: DEFAULT_PAGE_SIZE
			});
		},
		initialPageParam: 0,
		getNextPageParam(lastPage, _allPages, lastPageParam) {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		},
		enabled: !!subgraphUrl
	});

	const AppTable = TanstackAppTable<
		SgWithdrawal | SgDeposit | SgTradeVaultBalanceChange | SgClearBounty
	>;
</script>

<AppTable
	query={balanceChangesQuery}
	queryKey={undefined}
	emptyMessage="No deposits or withdrawals found"
	rowHoverable={false}
>
	<svelte:fragment slot="title">
		<Heading tag="h5" class="mb-4 mt-6 font-normal">Vault Balance Changes</Heading>
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
			{formatUnits(BigInt(item.amount), Number(item.vault.token.decimals ?? 0))}
			{item.vault.token.symbol}
		</TableBodyCell>
		<TableBodyCell tdClass="break-word p-0 text-left" data-testid="vaultBalanceChangesTableBalance">
			{formatUnits(BigInt(item.newVaultBalance), Number(item.vault.token.decimals ?? 0))}
			{item.vault.token.symbol}
		</TableBodyCell>
		<TableBodyCell tdClass="break-word p-0 text-left" data-testid="vaultBalanceChangesTableType">
			{item.__typename}
		</TableBodyCell>
	</svelte:fragment>
</AppTable>
