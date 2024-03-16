<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { walletDerivationIndex, walletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { walletconnectModal, account, orderbookAddress } from '$lib/stores/settings';
  import { orderRemove, orderRemoveCalldata } from '$lib/services/order';
  import { ethersExecute } from '$lib/services/ethersTx';
  import { get } from '@square/svelte-store';

  export let open = false;
  export let id: string;

  let isSubmitting = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: walletconnectLabel = $account
    ? `${$account.slice(0, 5)}...${$account.slice(-5)}`
    : "CONNECT"

  function reset() {
    open = false;
    selectedLedger = false;
    selectedWalletconnect = false;
  }

  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(id);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      const calldata = await orderRemoveCalldata(id) as Uint8Array;
      const tx = await ethersExecute(calldata, get(orderbookAddress)!)
      await tx.wait(1);
      reset();
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<Modal title="Remove Order" bind:open outsideclose size="sm" on:close={reset}>
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
      Remove Order
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
    <ButtonLoading on:click={executeWalletconnect} disabled={isSubmitting || !$account} loading={isSubmitting}>
      Remove Order
    </ButtonLoading>
  {/if}
</Modal>
