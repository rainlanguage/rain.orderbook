<script lang="ts">
  import { Button, TabItem, Tabs } from 'flowbite-svelte';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { BadgeActive, CardProperty } from '@rainlanguage/ui-components';
  import { formatTimestampSecondsAsLocal } from '@rainlanguage/ui-components';
  import {
    ButtonVaultLink,
    CodeMirrorRainlang,
    Hash,
    HashType,
    TanstackPageContentDetail,
    OrderTradesChart,
    OrderTradesListTable,
    OrderVaultsVolTable,
    TanstackOrderQuote,
    QKEY_ORDER,
  } from '@rainlanguage/ui-components';

  import { subgraphUrl } from '$lib/stores/settings';

  import { handleOrderRemoveModal } from '$lib/services/modal';
  import { createQuery } from '@tanstack/svelte-query';
  import { onDestroy } from 'svelte';
  import { queryClient } from '$lib/queries/queryClient';
  import { getOrder, type Order } from '@rainlanguage/orderbook/js_api';
  import { codeMirrorTheme } from '$lib/stores/darkMode';

  export let id: string;

  $: orderDetailQuery = createQuery<Order>({
    queryKey: [id, QKEY_ORDER + id],
    queryFn: () => getOrder($subgraphUrl || '', id),
    enabled: !!$subgraphUrl && !!id,
  });

  const interval = setInterval(async () => {
    // This invalidate function invalidates
    // both order detail and order trades list queries
    await queryClient.invalidateQueries({
      queryKey: [id],
      refetchType: 'active',
      exact: false,
    });
  }, 10000);

  onDestroy(() => {
    clearInterval(interval);
  });
</script>

<TanstackPageContentDetail query={orderDetailQuery} emptyMessage="Order not found">
  <svelte:fragment slot="top" let:data>
    <div class="flex gap-x-4 text-3xl font-medium dark:text-white">
      <div class="flex gap-x-2">
        <span class="font-light">Order</span>
        <Hash shorten value={data.orderHash} />
      </div>
      <BadgeActive active={data.active} large />
    </div>
    {#if data && $walletAddressMatchesOrBlank(data.owner) && data.active}
      <Button color="dark" on:click={() => handleOrderRemoveModal(data, $orderDetailQuery.refetch)}>
        Remove
      </Button>
    {/if}
  </svelte:fragment>
  <svelte:fragment slot="card" let:data>
    <div class="flex flex-col gap-y-6">
      <CardProperty>
        <svelte:fragment slot="key">Orderbook</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash type={HashType.Identifier} shorten={false} value={data.orderbook.id} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Owner</svelte:fragment>
        <svelte:fragment slot="value">
          <Hash type={HashType.Wallet} shorten={false} value={data.owner} />
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Created</svelte:fragment>
        <svelte:fragment slot="value">
          {formatTimestampSecondsAsLocal(BigInt(data.timestampAdded))}
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Input vaults</svelte:fragment>
        <svelte:fragment slot="value">
          <div class="mb-2 flex justify-end">
            <span>Balance</span>
          </div>
          <div class="space-y-2">
            {#each data.inputs || [] as t}
              <ButtonVaultLink tokenVault={t} />
            {/each}
          </div>
        </svelte:fragment>
      </CardProperty>

      <CardProperty>
        <svelte:fragment slot="key">Output vaults</svelte:fragment>
        <svelte:fragment slot="value">
          <div class="mb-2 flex justify-end">
            <span>Balance</span>
          </div>
          <div class="space-y-2">
            {#each data.outputs || [] as t}
              <ButtonVaultLink tokenVault={t} />
            {/each}
          </div>
        </svelte:fragment>
      </CardProperty>
    </div>
  </svelte:fragment>
  <svelte:fragment slot="chart">
    <OrderTradesChart {id} subgraphUrl={$subgraphUrl} />
  </svelte:fragment>
  <svelte:fragment slot="below" let:data>
    <TanstackOrderQuote {id} order={data} subgraphUrl={$subgraphUrl} />
    <Tabs
      style="underline"
      contentClass="mt-4"
      defaultClass="flex flex-wrap space-x-2 rtl:space-x-reverse mt-4"
    >
      <TabItem open title="Rainlang source">
        <div class="mb-8 overflow-hidden rounded-lg border dark:border-none">
          <CodeMirrorRainlang disabled={true} order={data} codeMirrorTheme={$codeMirrorTheme} />
        </div>
      </TabItem>
      <TabItem title="Trades">
        <OrderTradesListTable {id} subgraphUrl={$subgraphUrl} />
      </TabItem>
      <TabItem title="Volume">
        <OrderVaultsVolTable {id} subgraphUrl={$subgraphUrl} />
      </TabItem>
    </Tabs>
  </svelte:fragment>
</TanstackPageContentDetail>
