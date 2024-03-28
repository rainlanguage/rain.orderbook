<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import ButtonLoading from '$lib/components/ButtonLoading.svelte';
  import { ledgerWalletDerivationIndex, ledgerWalletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from './InputLedgerWallet.svelte';
  import { walletconnectModal, walletconnectAccount } from '$lib/stores/walletconnect';
  import { isNil } from 'lodash';

  export let open = false;
  export let title: string;
  export let execButtonLabel: string;
  export let executeLedger: () => Promise<void>;
  export let executeWalletconnect: () => Promise<void>;
  export let isSubmitting = false;
  export let onBack: (() => void) | undefined = undefined;

  let selectedLedger = false;
  let selectedWalletconnect = false;

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : "CONNECT"

  function reset() {
    open = false;
    if (!isSubmitting) {
      selectedLedger = false;
      selectedWalletconnect = false;
    }
  }
</script>

<Modal {title} bind:open outsideclose={!isSubmitting} size="sm" on:close={reset}>
  {#if !selectedLedger && !selectedWalletconnect}
    {#if onBack}
      <Button color="alternative" on:click={() => {onBack?.(); reset();}}>Back</Button>
    {/if}
    <div class="flex flex-col w-full justify-between space-y-2">
      <Button on:click={() => selectedLedger = true}>Ledger Wallet</Button>
      <Button on:click={() => selectedWalletconnect = true}>WalletConnect</Button>
    </div>
  {:else if selectedLedger}
    <Button color="alternative" on:click={() => selectedLedger = false}>Back</Button>
    <InputLedgerWallet
      bind:derivationIndex={$ledgerWalletDerivationIndex}
      bind:walletAddress={$ledgerWalletAddress.value}
    />
    <ButtonLoading class="w-full" on:click={() => executeLedger().finally(() => reset())} disabled={isSubmitting || !$ledgerWalletAddress || isNil($ledgerWalletDerivationIndex) || isNil($ledgerWalletDerivationIndex)} loading={isSubmitting}>
      {execButtonLabel}
    </ButtonLoading>
  {:else if selectedWalletconnect}
    <Button color="alternative" on:click={() => selectedWalletconnect = false}>Back</Button>
    <div class="flex flex-col w-full justify-between space-y-2">
      <div class="text-lg text-center">Only <b>mobile</b> wallets are supported in WalletConnect.</div>
      <Button
        color="blue"
        class="px-2 py-1"
        size="xs"
        pill
        on:click={() => $walletconnectModal?.open()}
      >
      {walletconnectLabel}
      </Button>
      <ButtonLoading on:click={() => executeWalletconnect().finally(() => reset())} disabled={isSubmitting || !$walletconnectAccount} loading={isSubmitting}>
        {execButtonLabel}
      </ButtonLoading>
    </div>
  {/if}
</Modal>
