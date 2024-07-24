<script lang="ts">
  import VaultBalanceChangesTable from '../tables/VaultBalanceChangesTable.svelte';
  import { Button } from 'flowbite-svelte';
  import { walletAddressMatchesOrBlank } from '$lib/stores/wallets';
  import { bigintStringToHex } from '$lib/utils/hex';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import { goto } from '$app/navigation';
  import { ArrowDownOutline, ArrowUpOutline } from 'flowbite-svelte-icons';
  import CardProperty from '$lib/components/CardProperty.svelte';
  import { formatUnits } from 'viem';
  import { createQuery } from '@tanstack/svelte-query';
  import { vaultDetail } from '$lib/queries/vaultDetail';
  import { QKEY_VAULT } from '$lib/queries/keys';
  import { subgraphUrl } from '$lib/stores/settings';
  import { handleDepositModal, handleWithdrawModal } from '$lib/services/modal';
  import TanstackContentDetail from '$lib/components/detail/TanstackPageContentDetail.svelte';
  import VaultBalanceChart from '$lib/components/charts/VaultBalanceChart.svelte';

  export let id: string;

  $: vaultDetailQuery = createQuery({
    queryKey: [QKEY_VAULT + id],
    queryFn: () => {
      return vaultDetail(id, $subgraphUrl || '');
    },
    enabled: !!$subgraphUrl,
  });
</script>

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
      <VaultBalanceChart vault={data} />
    {/if}
  </svelte:fragment>

  <svelte:fragment slot="below"><VaultBalanceChangesTable {id} /></svelte:fragment>
</TanstackContentDetail>
