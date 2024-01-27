<script lang="ts">
  import { Button, Modal, Label, ButtonGroup, Spinner, Helper } from 'flowbite-svelte';
  import type { TokenVault } from '$lib/typeshare/vault';
  import InputTokenAmount from '$lib/InputTokenAmount.svelte';
  import { vaultDeposit } from '$lib/utils/vaultDeposit';

  export let open = false;
  export let vault: TokenVault;
  let amount: string = '';
  let amountRaw: bigint;
  let isSubmitting = false;

  function reset() {
    amount = '';
    amountRaw = 0n;
    isSubmitting = false;
    open = false;
  }

  async function execute() {
    isSubmitting = true;
    await vaultDeposit(vault.vault.vault_id, vault.token.id, amountRaw);
    reset();
    isSubmitting = false;
  }
</script>

<Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={reset}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Vault ID
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.vault.vault_id}
    </p>
    <Helper class="mt-2 text-sm">
      A hex identifier to distinguish this Vault from others with the same Token and Owner
    </Helper>
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
      Owner
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.owner.id}
    </p>
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Balance
    </h5>
    <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
      {vault.balance_display}
    </p>
  </div>

  <div class="mb-6">
    <Label
      for="amount"
      class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
    >
      Amount
    </Label>
    <ButtonGroup class="w-full">
      <InputTokenAmount
        bind:value={amount}
        bind:valueRaw={amountRaw}
        symbol={vault.token.symbol}
        decimals={vault.token.decimals}
      />
    </ButtonGroup>
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button on:click={execute} disabled={!amountRaw || amountRaw === 0n || isSubmitting}>
        {#if isSubmitting}
          <Spinner class="mr-2 h-4 w-4" color="white" />
        {/if}
        Submit Deposit
      </Button>
    </div>
  </svelte:fragment>
</Modal>
