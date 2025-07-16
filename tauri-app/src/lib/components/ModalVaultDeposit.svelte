<script lang="ts">
  import { Button, Modal, Label, ButtonGroup } from 'flowbite-svelte';
  import type { RaindexVault } from '@rainlanguage/orderbook';
  import { vaultDeposit } from '$lib/services/vault';
  import { InputTokenAmount } from '@rainlanguage/ui-components';
  import { ethersExecute, checkERC20Balance } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { formatUnits, hexToBytes, toHex } from 'viem';
  import { onMount } from 'svelte';

  export let open = false;
  export let vault: RaindexVault;
  export let onDeposit: () => void;
  let amount: bigint;
  let isSubmitting = false;
  let selectWallet = false;
  let userBalance: bigint = 0n;

  function reset() {
    open = false;
    if (!isSubmitting) {
      amount = 0n;
      selectWallet = false;
    }
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultDeposit(vault.vaultId, vault.token.address, amount);
      onDeposit();
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }

  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const allowance = await vault.getAllowance();
      if (allowance.error) {
        throw new Error(allowance.error.readableMsg);
      }
      if (BigInt(allowance.value) < amount) {
        const calldata = await vault.getApprovalCalldata(amount.toString());
        if (calldata.error) {
          throw new Error(calldata.error.readableMsg);
        }
        const approveTx = await ethersExecute(hexToBytes(calldata.value), vault.token.address);
        toasts.success('Approve Transaction sent successfully!');
        await approveTx.wait(1);
      }

      const calldata = await vault.getDepositCalldata(amount.toString());
      if (calldata.error) {
        throw new Error(calldata.error.readableMsg);
      }
      const depositTx = await ethersExecute(hexToBytes(calldata.value), vault.orderbook);
      toasts.success('Transaction sent successfully!');
      await depositTx.wait(1);
      onDeposit();
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }

  async function fetchUserBalance() {
    try {
      userBalance = (await checkERC20Balance(vault.token.address)).toBigInt();
    } catch (_e) {
      userBalance = 0n;
    }
  }

  onMount(() => {
    fetchUserBalance();
  });
</script>

{#if !selectWallet}
  <Modal title="Deposit to Vault" bind:open outsideclose size="sm" on:close={reset}>
    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Vault ID
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {toHex(vault.vaultId)}
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
        Owner
      </h5>
      <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
        {vault.owner}
      </p>
    </div>

    <div class="flex justify-between">
      <div class="w-1/2">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Your Balance
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {formatUnits(userBalance, Number(vault.token.decimals ?? 0))}
        </p>
      </div>
      <div class="w-1/2">
        <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
          Vault Balance
        </h5>
        <p class="break-all font-normal leading-tight text-gray-700 dark:text-gray-400">
          {vault.formattedBalance}
        </p>
      </div>
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
          symbol={vault.token.symbol}
          decimals={Number(vault.token.decimals) ?? 0}
          maxValue={userBalance}
        />
      </ButtonGroup>
    </div>
    <div class="flex w-full justify-end space-x-4">
      <Button color="alternative" on:click={reset} disabled={isSubmitting}>Cancel</Button>
      <Button
        on:click={() => {
          selectWallet = true;
          open = false;
        }}
        disabled={!amount || amount === 0n || isSubmitting}
      >
        Proceed
      </Button>
    </div>
  </Modal>
{/if}

<ModalExecute
  chainId={vault.chainId}
  bind:open={selectWallet}
  onBack={() => (open = true)}
  title="Deposit to Vault"
  execButtonLabel="Deposit"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
