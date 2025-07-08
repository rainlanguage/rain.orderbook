<script lang="ts">
  import { Button, Modal, Label } from 'flowbite-svelte';
  import {
    vaultDeposit,
    vaultDepositApproveCalldata,
    vaultDepositCalldata,
  } from '$lib/services/vault';
  import { InputToken, InputTokenAmount, DropdownRadio } from '@rainlanguage/ui-components';
  import InputVaultId from '$lib/components/InputVaultId.svelte';
  import { checkAllowance, ethersExecute } from '$lib/services/ethersTx';
  import { toasts } from '$lib/stores/toasts';
  import ModalExecute from './ModalExecute.svelte';
  import { formatEthersTransactionError } from '$lib/utils/transaction';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { walletconnectAccount } from '$lib/stores/walletconnect';
  import { getOrderbookByChainId } from '$lib/utils/getOrderbookByChainId';
  import { settings } from '$lib/stores/settings';
  import { getNetworkName } from '@rainlanguage/ui-components';

  export let open = false;
  let networks = $settings.orderbook.networks;
  let vaultId: bigint | undefined = undefined;
  let tokenAddress: string = '';
  let tokenDecimals: number | undefined = undefined;
  let amount: bigint | undefined = undefined;
  let isSubmitting = false;
  let selectWallet = false;
  let chainId = 0;
  let selectedNetworkString = '';

  // Create network options for dropdown
  $: networkOptions = Object.entries(networks).reduce(
    (acc, [_key, network]) => {
      acc[network.chainId] = getNetworkName(network.chainId) || '';
      return acc;
    },
    {} as Record<string, string>,
  );

  // Update chainId when selected network changes
  $: if (selectedNetworkString) {
    chainId = parseInt(selectedNetworkString);
  }

  function reset() {
    open = false;
    if (!isSubmitting) {
      vaultId = undefined;
      tokenAddress = '';
      tokenDecimals = 0;
      amount = undefined;
      selectWallet = false;
    }
  }

  async function executeLedger() {
    if (vaultId === undefined) return;
    if (amount === undefined) return;

    isSubmitting = true;
    try {
      await vaultDeposit(vaultId, tokenAddress, amount);
    } catch (e) {
      reportErrorToSentry(e);
    }
    isSubmitting = false;
    reset();
  }

  async function executeWalletconnect() {
    if (vaultId === undefined) return;
    if (amount === undefined) return;
    if (!walletconnectAccount) {
      toasts.error('Connect your wallet to create a new vault');
      return;
    }

    isSubmitting = true;
    try {
      const orderbook = getOrderbookByChainId(chainId);

      const allowance = await checkAllowance(tokenAddress, orderbook.address);
      if (allowance.lt(amount)) {
        const approveCalldata = (await vaultDepositApproveCalldata(
          vaultId,
          tokenAddress,
          amount,
        )) as Uint8Array;
        const approveTx = await ethersExecute(approveCalldata, tokenAddress);
        toasts.success('Approve Transaction sent successfully!');
        await approveTx.wait(1);
      }

      const depositCalldata = (await vaultDepositCalldata(
        vaultId,
        tokenAddress,
        amount,
      )) as Uint8Array;
      const depositTx = await ethersExecute(depositCalldata, orderbook.address);
      toasts.success('Transaction sent successfully!');
      await depositTx.wait(1);
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(formatEthersTransactionError(e));
    }
    isSubmitting = false;
    reset();
  }
</script>

{#if !selectWallet}
  <Modal title="Deposit to Vault" bind:open outsideclose={!isSubmitting} size="sm" on:close={reset}>
    <div>
      <h5 class="mb-2 w-full text-xl font-bold tracking-tight text-gray-900 dark:text-white">
        Network
      </h5>
      <DropdownRadio options={networkOptions} bind:value={selectedNetworkString}>
        <svelte:fragment slot="content" let:selectedRef>
          <span>{selectedRef ? networkOptions[selectedRef] : 'Select a network'}</span>
        </svelte:fragment>
        <svelte:fragment slot="option" let:ref>
          <div class="w-full overflow-hidden overflow-ellipsis">
            <div class="text-md break-word mb-2">{networkOptions[ref]}</div>
          </div>
        </svelte:fragment>
      </DropdownRadio>
    </div>

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
  {chainId}
  bind:open={selectWallet}
  onBack={() => (open = true)}
  title="Deposit to Vault"
  execButtonLabel="Deposit"
  {executeLedger}
  {executeWalletconnect}
  bind:isSubmitting
/>
