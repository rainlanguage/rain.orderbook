<script lang="ts">
  import { Button, Modal, Label, Helper } from 'flowbite-svelte';
  import type { Vault } from '$lib/typeshare/vault';
  import InputTokenAmount from './InputTokenAmount.svelte';

  export let open = false;
  export let vault: Vault;
  let amount: string = '';
  let amountRaw: bigint = 0n;
  let amountGTBalance: boolean;

  $: {
    if (vault.token_vaults) {
      console.log('amount raw is ', amountRaw);
      try {
        amountGTBalance = amountRaw > vault.token_vaults[0].balance;
      } catch (e) {}
    }
  }

  function reset() {
    amount = '';
    amountRaw = 0n;
  }
</script>

<Modal title="Withdraw from Vault" bind:open outsideclose size="sm" on:close={reset}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Vault ID
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.id}
    </p>
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Token
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.token_vaults && vault.token_vaults[0].token.name}
    </p>
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Balance
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.token_vaults && vault.token_vaults[0].balance_display}
    </p>
  </div>

  <div class="mb-6">
    <Label
      for="amount"
      class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
    >
      Amount
    </Label>
    <InputTokenAmount
      bind:value={amount}
      bind:valueRaw={amountRaw}
      symbol={vault.token_vaults ? vault.token_vaults[0].token.symbol : ''}
      decimals={vault.token_vaults ? vault.token_vaults[0].token.decimals : 16}
    />

    <Helper color="red" class="h-6 text-sm">
      {#if amountGTBalance}
        Amount cannot exceed available balance.
      {/if}
    </Helper>
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={() => (open = false)}>Cancel</Button>

      <Button
        on:click={() => alert('Handle "success"')}
        disabled={!amountRaw || amountRaw === 0n || amountGTBalance}>Make Withdrawal</Button
      >
    </div>
  </svelte:fragment>
</Modal>
