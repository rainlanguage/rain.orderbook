<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import InputWalletConnect from '$lib/components/InputWalletConnect.svelte';
  import { walletConnectNetwork, walletconnectAccount } from '$lib/stores/walletconnect';
  import {
    IconWalletConnect,
    ButtonLoading,
    getNetworkName,
  } from '@rainlanguage/ui-components';
  import type { NetworkCfg } from '@rainlanguage/orderbook';

  export let open = false;
  export let title: string;
  export let execButtonLabel: string;
  export let executeWalletconnect: () => Promise<void>;
  export let isSubmitting = false;
  export let onBack: (() => void) | undefined = undefined;
  export let chainId: number | undefined = undefined;
  export let overrideNetwork: NetworkCfg | undefined = undefined;

  let selectedWalletconnect = false;

  function reset() {
    open = false;
    if (!isSubmitting) {
      selectedWalletconnect = false;
    }
  }
</script>

<Modal {title} bind:open outsideclose={!isSubmitting} size="sm" on:close={reset}>
  {#if !selectedWalletconnect && !$walletconnectAccount}
    <div class="flex justify-center space-x-4">
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
  {:else}
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
          You are connected to {getNetworkName($walletConnectNetwork) || 'an unknown'} network. Please
          connect your wallet to {overrideNetwork?.key || getNetworkName(chainId ?? 0) || 'unknown'}
          network.
        </div>
      {/if}
    </div>
  {/if}
</Modal>
