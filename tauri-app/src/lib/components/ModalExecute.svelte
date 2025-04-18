<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import { settings } from '$lib/stores/settings';
  import { ledgerWalletAddress } from '$lib/stores/wallets';
  import InputLedgerWallet from '$lib/components/InputLedgerWallet.svelte';
  import InputWalletConnect from '$lib/components/InputWalletConnect.svelte';
  import { walletConnectNetwork, walletconnectAccount } from '$lib/stores/walletconnect';
  import { IconLedger, IconWalletConnect, ButtonLoading } from '@rainlanguage/ui-components';
  import { activeNetworkRef, chainId as globalChainId } from '$lib/stores/settings';
  import type { NetworkCfg } from '@rainlanguage/orderbook';

  export let open = false;
  export let title: string;
  export let execButtonLabel: string;
  export let executeLedger: () => Promise<void>;
  export let executeWalletconnect: () => Promise<void>;
  export let isSubmitting = false;
  export let onBack: (() => void) | undefined = undefined;

  export let overrideNetwork: NetworkCfg | undefined = undefined;
  $: chainId = overrideNetwork?.chainId || $globalChainId;

  let selectedLedger = false;
  let selectedWalletconnect = false;

  function reset() {
    open = false;
    if (!isSubmitting) {
      selectedLedger = false;
      selectedWalletconnect = false;
    }
  }

  const getNetworkName = (chainId: number) => {
    const existingNetwork = Object.entries($settings?.networks || {}).find(
      (entry) => entry[1].chainId === chainId,
    );

    if (existingNetwork) {
      return existingNetwork[0];
    }

    return 'an unknown';
  };
</script>

<Modal {title} bind:open outsideclose={!isSubmitting} size="sm" on:close={reset}>
  {#if !selectedLedger && !selectedWalletconnect && !$walletconnectAccount && !$ledgerWalletAddress}
    <div class="flex justify-center space-x-4">
      <Button class="text-lg" on:click={() => (selectedLedger = true)}>
        <div class="mr-4">
          <IconLedger />
        </div>
        Ledger Wallet
      </Button>
      <Button class="text-lg" on:click={() => (selectedWalletconnect = true)}>
        <div class="mr-3">
          <IconWalletConnect />
        </div>
        WalletConnect
      </Button>
    </div>

    <div class="flex justify-end space-x-4">
      {#if onBack}
        <Button
          color="alternative"
          on:click={() => {
            onBack?.();
            reset();
          }}>Back</Button
        >
      {/if}
    </div>
  {:else if selectedLedger || $ledgerWalletAddress}
    <InputLedgerWallet />
    <div
      class={!$ledgerWalletAddress
        ? 'flex justify-between space-x-4'
        : 'flex justify-end space-x-4'}
    >
      {#if !$ledgerWalletAddress}
        <Button color="alternative" on:click={() => (selectedLedger = false)}>Back</Button>
      {/if}
      <ButtonLoading
        on:click={() => executeLedger().finally(() => reset())}
        disabled={isSubmitting || !$ledgerWalletAddress}
        loading={isSubmitting}
      >
        {execButtonLabel}
      </ButtonLoading>
    </div>
  {:else if selectedWalletconnect || $walletconnectAccount}
    <InputWalletConnect priorityChainIds={chainId ? [chainId] : []} />
    <div
      class={!$walletconnectAccount
        ? 'flex items-center justify-between space-x-4'
        : 'flex items-center justify-end space-x-4'}
    >
      {#if !$walletconnectAccount}
        <Button color="alternative" on:click={() => (selectedWalletconnect = false)}>Back</Button>
      {/if}
      <ButtonLoading
        on:click={() => executeWalletconnect().finally(() => reset())}
        disabled={isSubmitting || !$walletconnectAccount || $walletConnectNetwork !== chainId}
        loading={isSubmitting}
      >
        {execButtonLabel}
      </ButtonLoading>
      {#if $walletconnectAccount && $walletConnectNetwork !== chainId}
        <div class="text-red-500" data-testid="network-connection-error">
          You are connected to {getNetworkName($walletConnectNetwork)} network. Please connect your wallet
          to {overrideNetwork?.key || $activeNetworkRef} network.
        </div>
      {/if}
    </div>
  {/if}
</Modal>
