<script lang="ts">
  import VaultBalanceChangesTable from '../tables/VaultBalanceChangesTable.svelte';
  import { Button } from 'flowbite-svelte';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { bigintStringToHex } from '@rainlanguage/ui-components';
  import { Hash, HashType } from '@rainlanguage/ui-components';

  import { goto } from '$app/navigation';
  import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
  import { CardProperty } from '@rainlanguage/ui-components';
  import { formatUnits } from 'viem';
  import { createQuery } from '@tanstack/svelte-query';
  import { vaultDetail } from '$lib/queries/vaultDetail';
  import { QKEY_VAULT } from '@rainlanguage/ui-components';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import TanstackContentDetail from '$lib/components/detail/TanstackPageContentDetail.svelte';
  import VaultBalanceChart from '$lib/components/charts/VaultBalanceChart.svelte';
  import { onDestroy } from 'svelte';
  import { queryClient } from '$lib/queries/queryClient';
  import { settings } from '$lib/stores/settings';

  export let id: string;
  export let network: string;
  const subgraphUrl = $settings?.subgraphs?.[network] || '';

  $: vaultDetailQuery = createQuery({
    queryKey: [id, QKEY_VAULT + id],
    queryFn: () => {
      return vaultDetail(id, subgraphUrl || '');
    },
    enabled: !!subgraphUrl,
  });

  const interval = setInterval(async () => {
    // This invalidate function invalidates
    // both vault detail and vault balance changes queries
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

<TanstackContentDetail query={vaultDetailQuery} emptyMessage="Vault not found">
  <svelte:fragment slot="top" let:data>
    <div
      data-testid="vaultDetailTokenName"
      class="flex gap-x-4 text-3xl font-medium dark:text-white"
    >
      {data.token.name}
    </div>
    <div>
      {#if $walletAddressMatchesOrBlank(data.owner)}
        <Button
          data-testid="vaultDetailDepositButton"
          color="dark"
          on:click={() => handleDepositModal(data, $vaultDetailQuery.refetch)}
          ><ArrowDownOutline size="xs" class="mr-2" />Deposit</Button
        >
        <Button
          data-testid="vaultDetailWithdrawButton"
          color="dark"
          on:click={() => handleWithdrawModal(data, $vaultDetailQuery.refetch)}
          ><ArrowUpOutline size="xs" class="mr-2" />Withdraw</Button
        >
      {/if}
    </div>
  </svelte:fragment>
  <svelte:fragment slot="card" let:data>
    <CardProperty data-testid="vaultDetailVaultId">
      <svelte:fragment slot="key">Vault ID</svelte:fragment>
      <svelte:fragment slot="value">{bigintStringToHex(data.vaultId)}</svelte:fragment>
    </CardProperty>

    <CardProperty data-testid="vaultDetailOrderbookAddress">
      <svelte:fragment slot="key">Orderbook</svelte:fragment>
      <svelte:fragment slot="value">
        <Hash type={HashType.Identifier} value={data.orderbook.id} />
      </svelte:fragment>
    </CardProperty>

    <CardProperty data-testid="vaultDetailOwnerAddress">
      <svelte:fragment slot="key">Owner Address</svelte:fragment>
      <svelte:fragment slot="value">
        <Hash type={HashType.Wallet} value={data.owner} />
      </svelte:fragment>
    </CardProperty>

    <CardProperty data-testid="vaultDetailTokenAddress">
      <svelte:fragment slot="key">Token address</svelte:fragment>
      <svelte:fragment slot="value">
        <Hash value={data.token.id} />
      </svelte:fragment>
    </CardProperty>

    <CardProperty data-testid="vaultDetailBalance">
      <svelte:fragment slot="key">Balance</svelte:fragment>
      <svelte:fragment slot="value"
        >{formatUnits(BigInt(data.balance), Number(data.token.decimals ?? 0))}
        {data.token.symbol}</svelte:fragment
      >
    </CardProperty>

    <CardProperty>
      <svelte:fragment slot="key">Orders as input</svelte:fragment>
      <svelte:fragment slot="value">
        <p data-testid="vaultDetailOrdersAsInput" class="flex flex-wrap justify-start">
          {#if data.ordersAsInput && data.ordersAsInput.length > 0}
            {#each data.ordersAsInput as order}
              <Button
                class={'mr-1 mt-1 px-1 py-0' + (!order.active ? ' opacity-50' : '')}
                color={order.active ? 'green' : 'yellow'}
                data-order={order.id}
                data-testid={'vaultDetailOrderAsInputOrder' + order.id}
                on:click={() => goto(`/orders/${order.id}`)}
              >
                <Hash type={HashType.Identifier} value={order.orderHash} copyOnClick={false} />
              </Button>
            {/each}
          {:else}
            None
          {/if}
        </p>
      </svelte:fragment>
    </CardProperty>

    <CardProperty>
      <svelte:fragment slot="key">Orders as output</svelte:fragment>
      <svelte:fragment slot="value">
        <p data-testid="vaulDetailOrdersAsOutput" class="flex flex-wrap justify-start">
          {#if data.ordersAsOutput && data.ordersAsOutput.length > 0}
            {#each data.ordersAsOutput as order}
              <Button
                class={'mr-1 mt-1 px-1 py-0' + (!order.active ? ' opacity-50' : '')}
                color={order.active ? 'green' : 'yellow'}
                data-order={order.id}
                data-testid={'vaultDetailOrderAsOutputOrder' + order.id}
                on:click={() => goto(`/orders/${order.id}`)}
              >
                <Hash type={HashType.Identifier} value={order.orderHash} copyOnClick={false} />
              </Button>
            {/each}
          {:else}
            None
          {/if}
        </p>
      </svelte:fragment>
    </CardProperty>
  </svelte:fragment>

  <svelte:fragment slot="chart" let:data>
    <VaultBalanceChart vault={data} />
  </svelte:fragment>

  <svelte:fragment slot="below"><VaultBalanceChangesTable {id} /></svelte:fragment>
</TanstackContentDetail>
