<script lang="ts">
  import { Heading, Button, TableHeadCell, TableBodyCell } from 'flowbite-svelte';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import { bigintStringToHex } from '$lib/utils/hex';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { goto } from '$app/navigation';
  import { timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import { sortBy } from 'lodash';
  import { bigintToFloat } from '$lib/utils/number';
  import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
  import CardProperty from '$lib/components/CardProperty.svelte';
  import type { UTCTimestamp } from 'lightweight-charts';
  import { formatUnits } from 'viem';
  import { createInfiniteQuery, createQuery } from '@tanstack/svelte-query';
  import { vaultBalanceChangesList } from '$lib/queries/vaultBalanceChangesList';
  import { vaultDetail } from '$lib/queries/vaultDetail';
  import { QKEY_VAULT, QKEY_VAULT_CHANGES } from '$lib/queries/keys';
  import { subgraphUrl } from '$lib/stores/settings';
  import { DEFAULT_PAGE_SIZE } from '$lib/queries/constants';
  import TanstackAppTable from '$lib/components/tables/TanstackAppTable.svelte';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import type { Vault } from '$lib/typeshare/vaultDetail';
  import LightweightChartLine from '$lib/components/LightweightChartLine.svelte';
  import TanstackContentDetail from '$lib/components/TanstackPageContentDetail.svelte';

  $: balanceChangesQuery = createInfiniteQuery({
    queryKey: [QKEY_VAULT_CHANGES + $page.params.id],
    queryFn: ({ pageParam }) => {
      return vaultBalanceChangesList($page.params.id, $subgraphUrl || '', pageParam);
    },
    initialPageParam: 0,
    getNextPageParam(lastPage, _allPages, lastPageParam) {
      return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
    },
    refetchInterval: 10000,
    enabled: !!$subgraphUrl,
  });

  $: vaultDetailQuery = createQuery({
    queryKey: [QKEY_VAULT + $page.params.id],
    queryFn: () => {
      return vaultDetail($page.params.id, $subgraphUrl || '');
    },
    enabled: !!$subgraphUrl,
  });

  let vaultBalanceChangesChartData: { value: number; time: UTCTimestamp }[] = [];

  function prepareChartData(vault: Vault) {
    const transformedData = $balanceChangesQuery.data?.pages.flatMap((page) =>
      page.map((d) => ({
        value: bigintToFloat(BigInt(d.new_vault_balance), Number(vault.token.decimals ?? 0)),
        time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
      })),
    );

    return sortBy(transformedData, (d) => d.time);
  }

  $: if ($balanceChangesQuery.data?.pages.length && $vaultDetailQuery.data)
    vaultBalanceChangesChartData = prepareChartData($vaultDetailQuery.data);
</script>

<PageHeader title="Vault" />

<TanstackContentDetail query={vaultDetailQuery} emptyMessage="Vault not found">
  <svelte:fragment slot="top" let:data>
    <div class="flex gap-x-4 text-3xl font-medium dark:text-white">
      {data?.token.name}
    </div>
    <div>
      {#if data && $walletAddressMatchesOrBlank(data.owner)}
        <Button color="dark" on:click={() => handleDepositModal(data)}
          ><ArrowDownOutline size="xs" class="mr-2" />Deposit</Button
        >
        <Button color="dark" on:click={() => handleWithdrawModal(data)}
          ><ArrowUpOutline size="xs" class="mr-2" />Withdraw</Button
        >
      {/if}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="card" let:data>
    {#if data}
      <CardProperty>
        <svelte:fragment slot="key">Vault ID</svelte:fragment>
        <svelte:fragment slot="value">{bigintStringToHex(data.vault_id)}</svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Owner Address</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash type={HashType.Wallet} value={data.owner} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Token address</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash value={data.token.id} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Balance</svelte:fragment>
        <svelte:fragment slot="value"
          >{formatUnits(BigInt(data.balance), Number(data.token.decimals ?? 0))}
          {data.token.symbol}</svelte:fragment
        >
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Orders as input</svelte:fragment>
        <svelte:fragment slot="value">
          {#if data.orders_as_input && data.orders_as_input.length > 0}
            <p class="flex flex-wrap justify-start">
              {#each data.orders_as_input as order}
                <Button
                  class={'mr-1 mt-1 px-1 py-0' + (!order.active ? ' opacity-50' : '')}
                  color="light"
                  on:click={() => goto(`/orders/${order.id}`)}
                  ><Hash type={HashType.Identifier} value={order.id} copyOnClick={false} /></Button
                >
              {/each}
            </p>
          {:else}
            None
          {/if}
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Orders as output</svelte:fragment>
        <svelte:fragment slot="value">
          {#if data.orders_as_output && data.orders_as_output.length > 0}
            <p class="flex flex-wrap justify-start">
              {#each data.orders_as_output as order}
                <Button
                  class="mr-1 mt-1 px-1 py-0"
                  color="alternative"
                  on:click={() => goto(`/orders/${order.id}`)}
                  ><Hash type={HashType.Identifier} value={order.id} copyOnClick={false} /></Button
                >
              {/each}
            </p>
          {:else}
            None{/if}
        </svelte:fragment>
      </CardProperty>
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="chart" let:data>
    {#if data}
      <LightweightChartLine
        title="Balance history"
        priceSymbol={data.token.symbol}
        data={vaultBalanceChangesChartData}
        loading={$balanceChangesQuery.isLoading}
        emptyMessage="No deposits or withdrawals found"
      />
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="below">
    <TanstackAppTable
      query={balanceChangesQuery}
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
        <TableBodyCell tdClass="px-4 py-2">
          {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
        </TableBodyCell>
        <TableBodyCell tdClass="break-all py-2 min-w-48">
          <Hash type={HashType.Wallet} value={item.transaction.from} />
        </TableBodyCell>
        <TableBodyCell tdClass="break-all py-2 min-w-48">
          <Hash type={HashType.Transaction} value={item.transaction.id} />
        </TableBodyCell>
        <TableBodyCell tdClass="break-word p-0 text-left">
          {formatUnits(BigInt(item.amount), Number(item.vault.token.decimals ?? 0))}
          {item.vault.token.symbol}
        </TableBodyCell>
        <TableBodyCell tdClass="break-word p-0 text-left">
          {formatUnits(BigInt(item.new_vault_balance), Number(item.vault.token.decimals ?? 0))}
          {item.vault.token.symbol}
        </TableBodyCell>
        <TableBodyCell tdClass="break-word p-0 text-left">
          {item.__typename}
        </TableBodyCell>
      </svelte:fragment>
    </TanstackAppTable>
  </svelte:fragment>
</TanstackContentDetail>
