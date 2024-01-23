<script lang="ts">
  import { Button, Modal, Label, ButtonGroup } from 'flowbite-svelte';
  import type { TokenVault } from '$lib/typeshare/vault';
  import InputTokenAmount from '$lib/InputTokenAmount.svelte';

  export let open = false;
  export let vault: TokenVault;
  let amount: string = '';
  let amountRaw: bigint;

  function reset() {
    amount = '';
    amountRaw = 0n;
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
      <Button color="alternative" on:click={() => (open = false)}>Cancel</Button>
      <Button on:click={() => alert('Handle "success"')} disabled={!amountRaw || amountRaw === 0n}
        >Submit Deposit</Button
      >
    </div>
  </svelte:fragment>
</Modal>
