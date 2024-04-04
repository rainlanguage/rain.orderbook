<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { ledgerWalletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from '$lib/components/InputLedgerWallet.svelte';
  import InputWalletConnect from '$lib/components/InputWalletConnect.svelte';
  import { walletconnectAccount } from '$lib/stores/walletconnect';
  import IconLedger from './IconLedger.svelte';
  import IconWalletConnect from './IconWalletConnect.svelte';

  export let open = false;
  export let title: string;
  export let execButtonLabel: string;
  export let executeLedger: () => Promise<void>;
  export let executeWalletconnect: () => Promise<void>;
  export let isSubmitting = false;
  export let onBack: (() => void) | undefined = undefined;

  let selectedLedger = false;
  let selectedWalletconnect = false;

  function reset() {
    open = false;
    if (!isSubmitting) {
      selectedLedger = false;
      selectedWalletconnect = false;
    }
  }
</script>

<Modal {title} bind:open outsideclose={!isSubmitting} size="sm" on:close={reset}>
  {#if !selectedLedger && !selectedWalletconnect && !$walletconnectAccount && !$ledgerWalletAddress}
    <div class="flex justify-center space-x-4">
      <Button class="text-lg" on:click={() => selectedLedger = true}>
        <div class="mr-4">
          <IconLedger />
        </div>
        Ledger Wallet
      </Button>
      <Button class="text-lg" on:click={() => selectedWalletconnect = true}>
        <div class="mr-3">
          <IconWalletConnect />
        </div>
        WalletConnect
      </Button>
    </div>

    <div class="flex justify-end space-x-4">
      {#if onBack}
        <Button color="alternative" on:click={() => {onBack?.(); reset();}}>Back</Button>
      {/if}
    </div>
  {:else if selectedLedger || $ledgerWalletAddress}
    <InputLedgerWallet />
    <div class="flex justify-end space-x-4">
      <Button color="alternative" on:click={() => selectedLedger = false}>Back</Button>
      <ButtonLoading on:click={() => executeLedger().finally(() => reset())} disabled={isSubmitting || !$ledgerWalletAddress} loading={isSubmitting}>
        {execButtonLabel}
      </ButtonLoading>
    </div>
  {:else if selectedWalletconnect || $walletconnectAccount}
    <InputWalletConnect />
    <div class="flex justify-end space-x-4">
      {#if !$walletconnectAccount}
        <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
      {/if}
      <ButtonLoading on:click={() => executeWalletconnect().finally(() => reset())} disabled={isSubmitting || !$walletconnectAccount} loading={isSubmitting}>
        {execButtonLabel}
      </ButtonLoading>
    </div>
  {/if}
</Modal>
