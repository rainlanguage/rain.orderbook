<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { walletDerivationIndex, walletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { walletconnectModal, walletconnectAccount, orderbookAddress } from '$lib/stores/settings';
  import type { Deployment } from '$lib/typeshare/config';
  import { orderAdd, orderAddCalldata } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { get } from '@square/svelte-store';

  export let open = false;
  export let dotrainText: string;
  export let deployment: Deployment | undefined;
  let isSubmitting = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : "CONNECT"

  function reset() {
    open = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      if(!deployment) throw Error("Select a deployment to add order");

      await orderAdd(dotrainText, deployment);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      if(!deployment) throw Error("Select a deployment to add order");

      const calldata = await orderAddCalldata(dotrainText, deployment) as Uint8Array;
      const tx = await ethersExecute(calldata, get(orderbookAddress)!)
      await tx.wait(1);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<Modal title="Add Order" bind:open outsideclose size="sm" on:close={reset}>
  {#if !selectedLedger && !selectedWalletconnect}
    <div class="mb-6">
      <Button on:click={() => selectedLedger = true}>Ledger Wallet</Button>
      <Button on:click={() => selectedWalletconnect = true}>WalletConnect</Button>
    </div>
  {:else if selectedLedger}
    <Button color="alternative" on:click={() => selectedLedger = false}>Back</Button>
    <InputLedgerWallet
      bind:derivationIndex={$walletDerivationIndex}
      bind:walletAddress={$walletAddress.value}
    />
    <ButtonLoading on:click={executeLedger} disabled={isSubmitting || !$walletAddress || !$walletDerivationIndex} loading={isSubmitting}>
      Add Order
    </ButtonLoading>
  {:else if selectedWalletconnect}
    <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
    <Button
      color="blue"
      class="px-2 py-1"
      size="xs"
      pill
      on:click={() => $walletconnectModal?.open()}
    >
    {walletconnectLabel}
    </Button>
    <ButtonLoading on:click={executeWalletconnect} disabled={isSubmitting || !$walletconnectAccount} loading={isSubmitting}>
      Add Order
    </ButtonLoading>
  {/if}
</Modal>
