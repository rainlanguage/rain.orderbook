<script lang="ts">
  import { Button, Modal, Label, Input, InputAddon, ButtonGroup, Helper } from 'flowbite-svelte';
  import { parseUnits } from 'viem';
  import type { TokenVault } from '$lib/typeshare/vault';
  import { isStringValidNumber } from './utils/number';

  export let open = false;
  export let vault: TokenVault;
  let amount: string;

  $: amountIsValidNumber = amount && isStringValidNumber(amount);
  let amountRaw: bigint;

  $: {
    if (amount) {
      try {
        amountRaw = parseUnits(amount, vault.token.decimals);
      } catch (e) {}
    }
  }
</script>

<Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={() => (amount = '')}>
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
      <Input
        name="amount"
        bind:value={amount}
        on:keyup={() => (amount = amount.replace(/[^\d.]/g, ''))}
        placeholder="0"
      />
      <InputAddon>
        {vault.token.symbol}
      </InputAddon>
    </ButtonGroup>
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={() => (open = false)}>Cancel</Button>
      <Button on:click={() => alert('Handle "success"')} disabled={!amount || !amountIsValidNumber}
        >Submit Deposit</Button
      >
    </div>
  </svelte:fragment>
</Modal>
