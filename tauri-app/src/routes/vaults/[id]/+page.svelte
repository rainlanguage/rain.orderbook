<script lang="ts">
  import {
    Heading,
    Button,
    Card,
    TableHeadCell,
    TableBodyCell,
  } from 'flowbite-svelte';
  import { vaultDetail } from '$lib/stores/vaultDetail';
  import ModalVaultDeposit from '$lib/components/ModalVaultDeposit.svelte';
  import ModalVaultWithdraw from '$lib/components/ModalVaultWithdraw.svelte';
  import { walletAddressMatchesOrBlank } from '$lib/stores/settings';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import { page } from '$app/stores';
  import { useVaultListBalanceChanges } from '$lib/stores/vaultListBalanceChanges';
  import { bigintStringToHex } from '$lib/utils/hex';
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/utils/hash';
  import AppTable from '$lib/components/AppTable.svelte';
  import { goto } from '$app/navigation';
  import ChartHistogram from '$lib/components/ChartHistogram.svelte';
  import { timestampSecondsToUTCTimestamp } from '$lib/utils/time';
  import { sortBy } from 'lodash';
  import { VaultBalanceChangeType } from '$lib/types/vaultBalanceChange';

  let showDepositModal = false;
  let showWithdrawModal = false;

  vaultDetail.refetch($page.params.id);
  $: vault = $vaultDetail[$page.params.id];

  function toggleDepositModal() {
    showDepositModal = !showDepositModal;
  }
  function toggleWithdrawModal() {
    showWithdrawModal = !showWithdrawModal;
  }

  const vaultListBalanceChanges = useVaultListBalanceChanges($page.params.id);
  vaultListBalanceChanges.fetchAll(1);

  $: vaultListBalanceChangesAllChartData = $vaultListBalanceChanges.all.map((d) => ({
      value: d.type === VaultBalanceChangeType.Withdraw ? -1 * parseFloat(d.content.amount_display) : parseFloat(d.content.amount_display),
      time: timestampSecondsToUTCTimestamp(BigInt(d.content.timestamp)),
      color: d.type === VaultBalanceChangeType.Withdraw ? 'blue' : 'green'
  }));

  $: vaultListBalanceChangesAllChartDataSorted = sortBy(vaultListBalanceChangesAllChartData, (d) => d.time);
</script>

<PageHeader title="Vault">
  <svelte:fragment slot="actions">
    {#if vault && $walletAddressMatchesOrBlank(vault.owner.id)}
      <Button color="green" size="xs" on:click={toggleDepositModal}>Deposit</Button>
      <Button color="blue" size="xs" on:click={toggleWithdrawModal}>Withdraw</Button>
    {/if}
  </svelte:fragment>
</PageHeader>

{#if vault === undefined}
  <div class="text-center text-gray-900 dark:text-white">Vault not found</div>
{:else}
  <div class="flex w-full justify-center flex-wrap space-x-0 lg:flex-nowrap lg:space-x-4 ">
    <Card class="space-y-8 grow-0 w-full" size="md">
      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Vault ID
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {bigintStringToHex(vault.vault_id)}
        </p>
      </div>

      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Owner Address
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          <Hash type={HashType.Wallet} shorten={false} value={vault.owner.id} />
        </p>
      </div>

      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Token
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {vault.token.name}
        </p>
      </div>

      <div>
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Balance
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {vault.balance_display}
          {vault.token.symbol}
        </p>
      </div>

      {#if vault.orders && vault.orders.length > 0}
        <div>
          <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
            Orders
          </h5>
          <p class="flex flex-wrap justify-start">
            {#each vault.orders as order}
              <Button class="px-1 py-0 mt-1 mr-1" color="alternative" on:click={() => goto(`/orders/${order.id}`)}><Hash type={HashType.Identifier} value={order.id} copyOnClick={false} /></Button>
            {/each}
          </p>
        </div>
      {/if}
    </Card>

    <ChartHistogram data={vaultListBalanceChangesAllChartDataSorted} loading={$vaultListBalanceChanges.isFetchingAll} emptyMessage="No deposits or withdrawals found" />
  </div>

  <div class="space-y-12 mt-8">
    <div class="w-full">
      <Heading tag="h4" class="mb-2">Deposits & Withdrawals</Heading>

      <AppTable listStore={vaultListBalanceChanges} emptyMessage="No deposits or withdrawals found" rowHoverable={false}>
        <svelte:fragment slot="head">
          <TableHeadCell>Date</TableHeadCell>
          <TableHeadCell>Sender</TableHeadCell>
          <TableHeadCell>Transaction Hash</TableHeadCell>
          <TableHeadCell>Balance Change</TableHeadCell>
          <TableHeadCell>Type</TableHeadCell>
        </svelte:fragment>

        <svelte:fragment slot="bodyRow" let:item>
          <TableBodyCell tdClass="px-4 py-2">
            {formatTimestampSecondsAsLocal(BigInt(item.content.timestamp))}
          </TableBodyCell>
          <TableBodyCell tdClass="break-all py-2 min-w-48">
            <Hash type={HashType.Wallet} value={item.content.sender.id} />
          </TableBodyCell>
          <TableBodyCell tdClass="break-all py-2 min-w-48">
            <Hash type={HashType.Transaction} value={item.content.transaction.id} />
          </TableBodyCell>
          <TableBodyCell tdClass="break-word p-2 text-right">
            {item.type === VaultBalanceChangeType.Withdraw ? '-' : ''}{item.content.amount_display} {item.content.token_vault.token.symbol}
          </TableBodyCell>
          <TableBodyCell tdClass="break-word p-2 text-right">
            {item.type}
          </TableBodyCell>
        </svelte:fragment>
      </AppTable>
    </div>
  </div>
{/if}

<ModalVaultDeposit bind:open={showDepositModal} {vault} />

<ModalVaultWithdraw bind:open={showWithdrawModal} {vault} />
