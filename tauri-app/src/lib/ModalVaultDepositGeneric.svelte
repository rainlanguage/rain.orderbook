<script lang="ts">
  import { Button, Modal, Label, Spinner } from 'flowbite-svelte';
  import InputTokenAmount from '$lib/InputTokenAmount.svelte';
  import { vaultDeposit } from '$lib/utils/vaultDeposit';
  import InputToken from '$lib/InputToken.svelte';
  import InputVaultId from './InputVaultId.svelte';

  export let open = false;

  let vaultId: bigint = 0n;
  let tokenAddress: string = '';
  let tokenDecimals: number = 0;
  let amount: bigint;
  let isSubmitting = false;

  function reset() {
    vaultId = 0n;
    tokenAddress = '';
    tokenDecimals = 0;
    amount = 0n;
    isSubmitting = false;
    open = false;
  }

  async function execute() {
    isSubmitting = true;
    await vaultDeposit(vaultId, tokenAddress, amount);
    reset();
    isSubmitting = false;
  }
</script>

<Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={reset}>
  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Vault ID
    </h5>
    <InputVaultId bind:value={vaultId} />
  </div>

  <div>
    <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
      Token
    </h5>
    <InputToken bind:address={tokenAddress} bind:decimals={tokenDecimals} />
  </div>

  <div class="mb-6">
    <Label
      for="amount"
      class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white"
    >
      Amount
    </Label>
    <InputTokenAmount bind:value={amount} decimals={tokenDecimals} />
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button on:click={execute} disabled={!amount || amount === 0n || isSubmitting}>
        {#if isSubmitting}
          <Spinner class="mr-2 h-4 w-4" color="white" />
        {/if}
        Submit Deposit
      </Button>
    </div>
  </svelte:fragment>
</Modal>
