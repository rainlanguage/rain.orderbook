<script lang="ts">
  import { Alert } from 'flowbite-svelte';
  import { ButtonLoading, IconWarning, useRaindexClient } from '@rainlanguage/ui-components';
  import { Hash, HashType } from '@rainlanguage/ui-components';
  import {
    walletconnectConnect,
    walletconnectAccount,
    walletconnectDisconnect,
    walletconnectIsConnecting,
    walletconnectIsDisconnecting,
  } from '$lib/stores/walletconnect';

  const raindexClient = useRaindexClient();

  export let priorityChainIds: number[] | undefined = undefined;
  export let onConnect: () => void = () => {};

  const networks = raindexClient.getAllNetworks();
</script>

<Alert color="yellow" border class="mb-8">
  <IconWarning slot="icon" />
  Only mobile wallets are supported in WalletConnect.
</Alert>

{#if networks.error}
  <Alert color="red" border class="mb-8">
    <IconWarning slot="icon" />
    {networks.error.readableMsg}
  </Alert>
{:else}
  <div class="flex w-full justify-end space-x-2">
    <ButtonLoading
      color="primary"
      class="w-full px-2 py-1"
      size="lg"
      pill
      loading={$walletconnectIsDisconnecting || $walletconnectIsConnecting}
      on:click={() => {
        walletconnectConnect(networks.value, priorityChainIds ?? []).then(onConnect);
      }}
    >
      {#if $walletconnectAccount}
        <Hash type={HashType.Wallet} value={$walletconnectAccount} />
      {:else}
        Connect
      {/if}
    </ButtonLoading>
    {#if $walletconnectAccount}
      <ButtonLoading
        color="red"
        class="min-w-fit px-2 py-1"
        size="lg"
        pill
        loading={$walletconnectIsDisconnecting || $walletconnectIsConnecting}
        on:click={walletconnectDisconnect}
      >
        Disconnect
      </ButtonLoading>
    {/if}
  </div>
{/if}
