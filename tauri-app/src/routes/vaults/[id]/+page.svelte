<script lang="ts">
  import { Heading, Button, TableHeadCell, TableBodyCell } from 'flowbite-svelte';
  import { vaultDetail, useVaultBalanceChangesList } from '$lib/stores/vault';
  import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
  import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import { bigintStringToHex } from '$lib/utils/hex';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import { goto } from '$app/navigation';
  import LightweightChartHistogram from '$lib/components/LightweightChartHistogram.svelte';
  import { timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import { sortBy } from 'lodash';
  import { bigintToFloat } from '$lib/utils/number';
  import PageContentDetail from '$lib/components/PageContentDetail.svelte';
  import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
  import CardProperty from '$lib/components/CardProperty.svelte';
  import type { UTCTimestamp } from 'lightweight-charts';

  let showDepositModal = false;
  let showWithdrawModal = false;

  let vaultBalanceChangesChartData: { value: number; time: UTCTimestamp; color?: string }[] = [];
  const vaultBalanceChangesList = useVaultBalanceChangesList($page.params.id);

  function prepareChartData() {
    const transformedData = $vaultBalanceChangesList.all.map((d) => ({
      value:
        d.__typename === 'Withdrawal'
          ? bigintToFloat(BigInt(-1) * BigInt(d.amount), Number(vault.token.decimals ?? 0))
          : bigintToFloat(BigInt(d.amount), Number(vault.token.decimals ?? 0)),
      time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
      color: d.__typename === 'Withdrawal' ? '#4E4AF6' : '#046C4E',
    }));

    return sortBy(transformedData, (d) => d.time);
  }

  $: vault = $vaultDetail.data[$page.params.id];
  $: $vaultBalanceChangesList.all, (vaultBalanceChangesChartData = prepareChartData());

  vaultDetail.refetch($page.params.id);
  vaultBalanceChangesList.fetchAll(0);
</script>

<PageHeader title="Vault" />

<PageContentDetail
  isFetching={$vaultDetail.isFetching}
  isEmpty={vault === undefined}
  emptyMessage="Vault not found"
>
  <svelte:fragment slot="top">
    <div class="flex gap-x-4 text-3xl font-medium dark:text-white">
      {vault.token.name}
    </div>
    <div>
      {#if vault && $walletAddressMatchesOrBlank(vault.owner)}
        <Button color="dark" on:click={() => (showDepositModal = !showDepositModal)}
          ><ArrowDownOutline size="xs" class="mr-2" />Deposit</Button
        >
        <Button color="dark" on:click={() => (showWithdrawModal = !showWithdrawModal)}
          ><ArrowUpOutline size="xs" class="mr-2" />Withdraw</Button
        >
      {/if}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="card">
    <CardProperty>
      <svelte:fragment slot="key">Vault ID</svelte:fragment>
      <svelte:fragment slot="value">{bigintStringToHex(vault.vault_id)}</svelte:fragment>
    </CardProperty>

    <CardProperty>
      <svelte:fragment slot="key">Owner Address</svelte:fragment>
      <svelte:fragment slot="value">
        <Hash type={HashType.Wallet} value={vault.owner} />
      </svelte:fragment>
    </CardProperty>

    <CardProperty>
      <svelte:fragment slot="key">Token address</svelte:fragment>
      <svelte:fragment slot="value">
        <Hash value={vault.token.id} />
      </svelte:fragment>
    </CardProperty>

    <CardProperty>
      <svelte:fragment slot="key">Balance</svelte:fragment>
      <svelte:fragment slot="value">{vault.balance} {vault.token.symbol}</svelte:fragment>
    </CardProperty>

    <CardProperty>
      <svelte:fragment slot="key">Orders</svelte:fragment>
      <svelte:fragment slot="value">
        {#if vault.orders_as_output && vault.orders_as_output.length > 0}
          <p class="flex flex-wrap justify-start">
            {#each vault.orders_as_output as order}
              <Button
                class="mr-1 mt-1 px-1 py-0"
                color="alternative"
                on:click={() => goto(`/orders/${order.id}`)}
                ><Hash type={HashType.Identifier} value={order.id} copyOnClick={false} /></Button
              >
            {/each}
          </p>
        {/if}
      </svelte:fragment>
    </CardProperty>
  </svelte:fragment>

  <svelte:fragment slot="chart">
    <LightweightChartHistogram
      title="Deposits & Withdrawals"
      priceSymbol={vault.token.symbol}
      data={vaultBalanceChangesChartData}
      loading={$vaultBalanceChangesList.isFetchingAll}
      emptyMessage="No deposits or withdrawals found"
    />
  </svelte:fragment>

  <svelte:fragment slot="below">
    <Heading tag="h5" class="mb-4 mt-6 font-normal">Deposits & Withdrawals</Heading>

    <AppTable
      listStore={vaultBalanceChangesList}
      emptyMessage="No deposits or withdrawals found"
      rowHoverable={false}
    >
      <svelte:fragment slot="head">
        <TableHeadCell padding="p-4">Date</TableHeadCell>
        <TableHeadCell padding="p-0">Sender</TableHeadCell>
        <TableHeadCell padding="p-0">Transaction Hash</TableHeadCell>
        <TableHeadCell padding="p-0">Balance Change</TableHeadCell>
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
          {item.amount}
          {item.vault.token.symbol}
        </TableBodyCell>
        <TableBodyCell tdClass="break-word p-0 text-left">
          {item.__typename}
        </TableBodyCell>
      </svelte:fragment>
    </AppTable>
  </svelte:fragment>
</PageContentDetail>

<ModalVaultDeposit bind:open={showDepositModal} {vault} />
<ModalVaultWithdraw bind:open={showWithdrawModal} {vault} />
