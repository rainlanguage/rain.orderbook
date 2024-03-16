<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { walletDerivationIndex, walletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { walletconnectModal, account } from '$lib/stores/settings';
  import { orderRemove } from '$lib/services/order';

  export let open = false;
  export let id: string;

  let isSubmitting = false;
  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: label = $account
    ? `${$account.slice(0, 5)}...${$account.slice(-1 * 5)}`
    : "CONNECT"

  async function executeLedger() {
    isSubmitting = true;
    try {
      await orderRemove(id);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
  async function executeWalletconnect() {
    isSubmitting = true;
    try {
      await orderRemove(id);
      // eslint-disable-next-line no-empty
    } catch (e) {}
    isSubmitting = false;
  }
</script>

<Modal title="Remove Order" bind:open outsideclose size="sm">
  {#if !selectedLedger && !selectedWalletconnect}
    <div class="mb-6">
      <ButtonLoading on:click={() => selectedLedger = true} disabled={false} loading={isSubmitting}>
        Ledger Wallet
      </ButtonLoading>
      <ButtonLoading on:click={() => selectedWalletconnect = true} disabled={false} loading={isSubmitting}>
        WalletConnect
      </ButtonLoading>
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
    {label}
    </Button>
    <ButtonLoading on:click={executeWalletconnect} disabled={isSubmitting || !$account} loading={isSubmitting}>
      Remove Order
    </ButtonLoading>
  {/if}
</Modal>
