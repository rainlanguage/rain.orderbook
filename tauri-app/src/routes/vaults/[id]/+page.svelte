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
  import ArrowLeftSolid from 'flowbite-svelte-icons/ArrowLeftSolid.svelte';
  import { vaultDetail } from '$lib/stores/vaultDetail';
  import ModalVaultDeposit from '$lib/ModalVaultDeposit.svelte';
  import ModalVaultWithdraw from '$lib/ModalVaultWithdraw.svelte';
  import { walletAddress } from '$lib/stores/settings';
  import { toHex } from 'viem';
  import { formatTimestampSecondsAsLocal } from '$lib/utils/time';

  export let data: { id: string };
  let showDepositModal = false;
  let showWithdrawModal = false;

  vaultDetail.refetch(data.id);
  $: vault = $vaultDetail[data.id];

  function toggleDepositModal() {
    showDepositModal = !showDepositModal;
  }
  function toggleWithdrawModal() {
    showWithdrawModal = !showWithdrawModal;
  }
</script>

<div class="flex w-full">
  <div class="flex-1">
    <Button outline size="xs" class="w-32" color="primary" href="/vaults">
      <ArrowLeftSolid size="xs" /><span class="ml-2">All Vaults</span>
    </Button>
  </div>
  <h1 class="flex-0 mb-8 text-4xl font-bold text-gray-900 dark:text-white">Vault</h1>
  <div class="flex-1"></div>
</div>
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
          {toHex(vault.vault.vault_id)}
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

      {#if $walletAddress !== '' && vault.owner.id === $walletAddress}
        <div class="pt-4">
          <div class="flex justify-center space-x-20">
            <Button color="green" size="xl" on:click={toggleDepositModal}>Deposit</Button>
            <Button color="blue" size="xl" on:click={toggleWithdrawModal}>Withdraw</Button>
          </div>
        </div>
      {/if}
    </Card>

    <div class="max-w-screen-xl space-y-12">
      <div class="w-full">
        <Heading tag="h4" class="mb-2">Withdrawals</Heading>

        {#if !vault.vault.withdraws || vault.vault.withdraws.length === 0}
          <div class="my-4 text-center text-gray-900 dark:text-white">No withdrawals found</div>
        {:else}
          <Table divClass="mx-8 cursor-pointer" hoverable={true}>
            <TableHead>
              <TableHeadCell>Sender</TableHeadCell>
              <TableHeadCell>Requested Amount</TableHeadCell>
              <TableHeadCell>Amount</TableHeadCell>
            </TableHead>
            <TableBody>
              {#each vault.vault.withdraws as withdraw}
                <TableBodyRow>
                  <TableBodyCell tdClass="break-all px-4 py-2">{withdraw.sender.id}</TableBodyCell>
                  <TableBodyCell tdClass="break-word p-2"
                    >{withdraw.requested_amount_display}</TableBodyCell
                  >
                  <TableBodyCell tdClass="break-word p-2">{withdraw.amount_display}</TableBodyCell>
                </TableBodyRow>
              {/each}
            </TableBody>
          </Table>
        {/if}
      </div>

      <div class="w-full">
        <Heading tag="h4" class="mb-2">Deposits</Heading>

        {#if !vault.vault.deposits || vault.vault.deposits.length === 0}
          <div class="my-4 text-center text-gray-900 dark:text-white">No deposits found</div>
        {:else}
          <Table divClass="cursor-pointer" hoverable={true}>
            <TableHead>
              <TableHeadCell>Date</TableHeadCell>
              <TableHeadCell>Sender</TableHeadCell>

              <TableHeadCell>Amount</TableHeadCell>
            </TableHead>
            <TableBody>
              {#each vault.vault.deposits as deposit}
                <TableBodyRow>
                  <TableBodyCell tdClass="px-4 py-2">
                    {formatTimestampSecondsAsLocal(BigInt(deposit.timestamp))}
                  </TableBodyCell>
                  <TableBodyCell tdClass="break-all py-2 text-xs space-y-1">
                    {deposit.sender.id}
                  </TableBodyCell>
                  <TableBodyCell tdClass="break-word p-2 text-right"
                    >{deposit.amount_display}
                    {vault.token.symbol}</TableBodyCell
                  >
                </TableBodyRow>
              {/each}
            </TableBody>
          </Table>
        {/if}
      </div>
    </div>
  </div>
{/if}

<ModalVaultDeposit bind:open={showDepositModal} {vault} />

<ModalVaultWithdraw bind:open={showWithdrawModal} {vault} />
