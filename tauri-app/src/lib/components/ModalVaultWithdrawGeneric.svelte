<script lang="ts">
  import { Button, Modal, Label } from 'flowbite-svelte';
  import InputTokenAmount from '$lib/InputTokenAmount.svelte';
  import { vaultWithdraw } from '$lib/utils/vaultWithdraw';
  import InputToken from '$lib/components/InputToken.svelte';
  import InputVaultId from '$lib/components/InputVaultId.svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';

  export let open = false;

  let vaultId: bigint;
  let tokenAddress: string;
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
    try {
      await vaultWithdraw(vaultId, tokenAddress, amount);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<Modal title="Withdraw from Vault" bind:open outsideclose size="sm" on:close={reset}>
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
      Target Amount
    </Label>
    <InputTokenAmount bind:value={amount} decimals={tokenDecimals} />
  </div>

  <svelte:fragment slot="footer">
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <ButtonLoading on:click={execute} disabled={!amount || amount === 0n || isSubmitting} loading={isSubmitting}>
        Make Withdrawal
      </ButtonLoading>
    </div>
  </svelte:fragment>
</Modal>
