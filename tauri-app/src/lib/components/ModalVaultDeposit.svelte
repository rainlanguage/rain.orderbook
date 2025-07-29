<script lang="ts">
  import { Button, Modal, Label, ButtonGroup } from 'flowbite-svelte';
  import type { AccountBalance, RaindexVault } from '@rainlanguage/orderbook';
  import { vaultDeposit } from '$lib/services/vault';
  import { InputTokenAmount, useRaindexClient } from '@rainlanguage/ui-components';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { hexToBytes, parseUnits, toHex } from 'viem';
  import { onMount } from 'svelte';

  const raindexClient = useRaindexClient();

  export let open = false;
  export let vault: RaindexVault;
  export let onDeposit: () => void;

  let amount: string;
  let isSubmitting = false;
  let selectWallet = false;
  let userBalance: AccountBalance = {
    balance: BigInt(0),
    formattedBalance: '0',
  } as unknown as AccountBalance;

  function reset() {
    open = false;
    if (!isSubmitting) {
      amount = '0';
      selectWallet = false;
    }
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await vaultDeposit(raindexClient, vault, parseUnits(amount, Number(vault.token.decimals)));
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
      if (BigInt(allowance.value) < parseUnits(amount, Number(vault.token.decimals))) {
        const calldata = await vault.getApprovalCalldata(amount);
        if (calldata.error) {
          throw new Error(calldata.error.readableMsg);
        }
        const approveTx = await ethersExecute(hexToBytes(calldata.value), vault.token.address);
        toasts.success('Approve Transaction sent successfully!');
        await approveTx.wait(1);
      }

      const calldata = await vault.getDepositCalldata(amount);
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
    const balance = await vault.getOwnerBalance();
    if (balance.error) {
      throw new Error(balance.error.readableMsg);
    }
    userBalance = balance.value;
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
          {userBalance.formattedBalance}
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
          maxValue={userBalance.formattedBalance}
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
        disabled={!amount || amount === '0' || isSubmitting}
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
