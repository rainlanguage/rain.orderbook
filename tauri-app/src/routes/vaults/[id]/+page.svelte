<script lang="ts">
  import {
    Heading,
    Button,
    Card,
    Table,
    TableHead,
    TableHeadCell,
    TableBody,
    TableBodyRow,
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
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { FileCsvOutline } from 'flowbite-svelte-icons';
  import ButtonsPagination  from '$lib/components/ButtonsPagination.svelte';

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
  vaultListBalanceChanges.fetchFirst();
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
  <div class="flex w-full flex-wrap justify-evenly space-y-12 xl:space-x-8 2xl:space-y-0">
    <Card class="space-y-8" size="lg">
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
          {vault.owner.id}
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
        <p class="break-all break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {vault.balance_display}
          {vault.token.symbol}
        </p>
      </div>
    </Card>

    <div class="max-w-screen-xl space-y-12">
      <div class="w-full">
        <Heading tag="h4" class="mb-2">Deposits & Withdrawals</Heading>

        {#if $vaultListBalanceChanges.currentPage.length === 0}
          <div class="my-4 text-center text-gray-900 dark:text-white">No deposits or withdrawals found</div>
        {:else}
          <Table divClass="cursor-pointer">
            <TableHead>
              <TableHeadCell>Date</TableHeadCell>
              <TableHeadCell>Sender</TableHeadCell>
              <TableHeadCell>Transaction Hash</TableHeadCell>
              <TableHeadCell>Balance Change</TableHeadCell>
              <TableHeadCell>Type</TableHeadCell>
            </TableHead>
            <TableBody>
              {#each $vaultListBalanceChanges.currentPage as vaultBalanceChange}
                  <TableBodyRow>
                    <TableBodyCell tdClass="px-4 py-2">
                      {formatTimestampSecondsAsLocal(BigInt(vaultBalanceChange.content.timestamp))}
                    </TableBodyCell>
                    <TableBodyCell tdClass="break-all py-2 text-xs space-y-1">
                      {vaultBalanceChange.content.sender.id}
                    </TableBodyCell>
                    <TableBodyCell tdClass="break-all py-2 text-xs space-y-1">
                      {vaultBalanceChange.content.transaction.id}
                    </TableBodyCell>
                    <TableBodyCell tdClass="break-word p-2 text-right">
                      {vaultBalanceChange.type === 'Withdraw' ? '-' : ''}{vaultBalanceChange.content.amount_display} {vaultBalanceChange.content.token_vault.token.symbol}
                    </TableBodyCell>
                    <TableBodyCell tdClass="break-word p-2 text-right">
                      {vaultBalanceChange.type}
                    </TableBodyCell>
                  </TableBodyRow>
              {/each}
            </TableBody>
          </Table>

          <div class="flex justify-between mt-2">
            <ButtonLoading size="xs" color="blue" on:click={() => vaultListBalanceChanges.exportCsv()} loading={$vaultListBalanceChanges.isExporting}>
              <FileCsvOutline class="w-4 h-4 mr-2"/>
              Export to CSV
            </ButtonLoading>
            <ButtonsPagination index={$vaultListBalanceChanges.index} on:previous={vaultListBalanceChanges.fetchPrev} on:next={vaultListBalanceChanges.fetchNext} loading={$vaultListBalanceChanges.isFetching} />
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}

<ModalVaultDeposit bind:open={showDepositModal} {vault} />

<ModalVaultWithdraw bind:open={showWithdrawModal} {vault} />
