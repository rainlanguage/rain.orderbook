<script lang="ts">
  import IconWarning from '$lib/components/IconWarning.svelte';
  import { Alert } from 'flowbite-svelte';
  import { ButtonLoading } from '@rainlanguage/ui-components';
  import { Hash, HashType } from '@rainlanguage/ui-components';
  import {
    walletconnectConnect,
    walletconnectAccount,
    walletconnectDisconnect,
    walletconnectIsConnecting,
    walletconnectIsDisconnecting,
  } from '$lib/stores/walletconnect';

  export let priorityChainIds: number[] | undefined = undefined;
  export let onConnect: () => void = () => {};
</script>

<div>
  <Alert color="yellow" border class="mb-8">
    <IconWarning slot="icon" />
    Only mobile wallets are supported in WalletConnect.
  </Alert>

  <div class="flex w-full justify-end space-x-2">
    <ButtonLoading
      color="primary"
      class="w-full px-2 py-1"
      size="lg"
      pill
      loading={$walletconnectIsDisconnecting || $walletconnectIsConnecting}
      on:click={() => {
        walletconnectConnect(priorityChainIds ?? []).then(onConnect);
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
</div>
