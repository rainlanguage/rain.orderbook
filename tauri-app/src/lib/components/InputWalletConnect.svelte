<script lang="ts">
  import IconWarning from "$lib/components/IconWarning.svelte";
  import { Alert } from "flowbite-svelte";
  import ButtonLoading from "./ButtonLoading.svelte";
  import Hash from '$lib/components/Hash.svelte';
  import { HashType } from '$lib/types/hash';
  import {
    walletconnectConnect,
    walletconnectAccount,
    walletconnectDisconnect,
    walletconnectIsConnecting,
    walletconnectIsDisconnecting,
  } from '$lib/stores/walletconnect';
</script>

<div>
  <Alert color="yellow" border class="mb-8">
    <IconWarning slot="icon" />
    Only mobile wallets are supported in WalletConnect.
  </Alert>

  <div class="flex w-full justify-end space-x-2">
    <ButtonLoading
      color="primary"
      class="px-2 py-1 w-full"
      size="lg"
      pill
      loading={$walletconnectIsDisconnecting || $walletconnectIsConnecting}
      on:click={walletconnectConnect}
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
        class="px-2 py-1 min-w-fit"
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
