<script lang="ts">
  import { Heading, TableBodyCell, TableHeadCell } from 'flowbite-svelte';
  import { orderDetail } from '$lib/stores/orderDetail';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import BadgeActive from '$lib/components/BadgeActive.svelte';
  import { formatTimestampSecondsAsLocal, timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import ButtonVaultLink from '$lib/components/ButtonVaultLink.svelte';
  import { orderRemove } from '$lib/utils/orderRemove';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/utils/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import { sortBy } from 'lodash';
  import { useOrderTakesList } from '$lib/stores/orderTakesList';
  import LightweightChartLine from '$lib/components/LightweightChartLine.svelte';
  import PageContentDetail from '$lib/components/PageContentDetail.svelte';

  let isSubmitting = false;

  $: order = $orderDetail[$page.params.id];

  async function remove() {
    isSubmitting = true;
    try {
      await orderRemove(order.id);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }

  const orderTakesList = useOrderTakesList($page.params.id);
  orderTakesList.fetchAll(1);

  $: orderTakesListChartData = $orderTakesList.all.map((d) => ({
      value: parseFloat(d.ioratio),
      time: timestampSecondsToUTCTimestamp(BigInt(d.timestamp)),
      color: 'blue',
  }));
  $: orderTakesListChartDataSorted = sortBy(orderTakesListChartData, (d) => d.time);

  orderDetail.refetch($page.params.id);
</script>

<PageHeader title="Order">
  <svelte:fragment slot="actions">
    {#if order && $walletAddressMatchesOrBlank(order.owner.id) && order.order_active}
      <ButtonLoading color="blue" size="xs" on:click={remove} loading={isSubmitting}>
        Remove
      </ButtonLoading>
    {/if}
  </svelte:fragment>
</PageHeader>

<PageContentDetail item={order} emptyMessage="Order not found">
  <svelte:fragment slot="card">
    <BadgeActive active={order.order_active} class="absolute right-5 top-5"/>
    <div class="mt-4">
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Order ID
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        <Hash type={HashType.Identifier} shorten={false} value={order.id} />
      </p>
    </div>

    <div class="mt-8">
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Owner Address
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        <Hash type={HashType.Wallet} shorten={false} value={order.owner.id} />
      </p>
    </div>

    <div class="mt-8">
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Created At
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {formatTimestampSecondsAsLocal(BigInt(order.timestamp))}
      </p>
    </div>

    <div class="mt-8">
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Input Vaults
      </h5>
      <div class="flex flex-wrap space-x-2 space-y-2">
        {#each (order.valid_inputs || []) as t}
          <ButtonVaultLink tokenVault={t.token_vault} />
        {/each}
      </div>
    </div>

    <div class="mt-8">
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Output Vaults
      </h5>
      <div class="flex flex-wrap space-x-2 space-y-2">
        {#each (order.valid_outputs || []) as t}
          <ButtonVaultLink tokenVault={t.token_vault} />
        {/each}
      </div>
    </div>
  </svelte:fragment>

  <svelte:fragment slot="chart">
    <LightweightChartLine title="Takes" data={orderTakesListChartDataSorted} loading={$orderTakesList.isFetchingAll} emptyMessage="No takes found" />
  </svelte:fragment>

  <svelte:fragment slot="below">
    <Heading tag="h4" class="mb-2">Takes</Heading>

      <AppTable listStore={orderTakesList} emptyMessage="No takes found" rowHoverable={false}>
        <svelte:fragment slot="head">
          <TableHeadCell>Date</TableHeadCell>
          <TableHeadCell>Sender</TableHeadCell>
          <TableHeadCell>Transaction Hash</TableHeadCell>
          <TableHeadCell>Input</TableHeadCell>
          <TableHeadCell>Output</TableHeadCell>
          <TableHeadCell>IO Ratio</TableHeadCell>
        </svelte:fragment>

        <svelte:fragment slot="bodyRow" let:item>
          <TableBodyCell tdClass="px-4 py-2">
            {formatTimestampSecondsAsLocal(BigInt(item.timestamp))}
          </TableBodyCell>
          <TableBodyCell tdClass="break-all py-2 min-w-32">
            <Hash type={HashType.Wallet} value={item.sender.id} />
          </TableBodyCell>
          <TableBodyCell tdClass="break-all py-2 min-w-32">
            <Hash type={HashType.Transaction} value={item.transaction.id} />
          </TableBodyCell>
          <TableBodyCell tdClass="break-all py-2">
            {item.input_display} {item.input_token.symbol}
          </TableBodyCell>
          <TableBodyCell tdClass="break-all py-2">
            {item.output_display} {item.output_token.symbol}
          </TableBodyCell>
          <TableBodyCell tdClass="break-all py-2">
            {item.ioratio}  {item.input_token.symbol}/{item.output_token.symbol}
          </TableBodyCell>
        </svelte:fragment>
      </AppTable>
  </svelte:fragment>
</PageContentDetail>
