<script lang="ts">
  import { Button, Modal, Label } from 'flowbite-svelte';
  import InputTokenAmount from '$lib/components/InputTokenAmount.svelte';
  import { vaultDeposit, vaultDepositApproveCalldata, vaultDepositCalldata } from '$lib/services/vault';
  import InputToken from '$lib/components/InputToken.svelte';
  import InputVaultId from '$lib/components/InputVaultId.svelte';
  import { orderbookAddress } from '$lib/stores/settings';
  import { checkAllowance, ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';

  export let open = false;
  let vaultId: bigint = 0n;
  let tokenAddress: string = '';
  let tokenDecimals: number = 0;
  let amount: bigint;
  let isSubmitting = false;
  let selectWallet = false;

  function reset() {
    open = false;
    if (!isSubmitting) {
      vaultId = 0n;
      tokenAddress = '';
      tokenDecimals = 0;
      amount = 0n;
      selectWallet = false;
    }
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultDeposit(vaultId, tokenAddress, amount);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
    reset();
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      if (!$orderbookAddress) throw Error("Select an orderbook to deposit");
      const allowance = await checkAllowance(amount, tokenAddress, $orderbookAddress);
      if (!allowance) {
        const approveCalldata = await vaultDepositApproveCalldata(vaultId, tokenAddress, amount) as Uint8Array;
        const approveTx = await ethersExecute(approveCalldata, tokenAddress);
        toasts.success("Approve Transaction sent successfully!");
        await approveTx.wait(1);
      }

      const depositCalldata = await vaultDepositCalldata(vaultId, tokenAddress, amount) as Uint8Array;
      const depositTx = await ethersExecute(depositCalldata, $orderbookAddress);
      toasts.success("Transaction sent successfully!");
      await depositTx.wait(1);

    } catch (e) {
      // eslint-disable-next-line no-console
      console.log(e);
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if (typeof e === "object" && (e as any)?.reason) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        toasts.error(`Transaction failed, reason: ${(e as any).reason}`);
      }
      else if (typeof e === "string") toasts.error(e);
      else if (e instanceof Error) toasts.error(e.message);
      else toasts.error("Transaction failed!");
    }
    isSubmitting = false;
    reset();
  }
</script>

{#if !selectWallet}
  <Modal title="Deposit to Vault" bind:open outsideclose={!isSubmitting} size="sm" on:close={reset}>
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
      <InputTokenAmount bind:value={amount} bind:decimals={tokenDecimals} />
    </div>
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button on:click={() => {selectWallet = true; open = false;}} disabled={!amount || amount === 0n || isSubmitting}>
        Proceed
      </Button>
    </div>
  </Modal>
{/if}

<ModalExecute
  bind:open={selectWallet}
  onBack={() => open = true}
  title="Deposit to Vault"
  execButtonLabel="Deposit"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting={isSubmitting}
/>