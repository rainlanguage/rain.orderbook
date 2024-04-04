<script lang="ts">
  import IconWarning from "$lib/components/IconWarning.svelte";
  import { Alert } from "flowbite-svelte";
  import ButtonLoading from "./ButtonLoading.svelte";
  import { walletconnectConnect, walletconnectIsDisconnecting, walletconnectAccount, walletconnectIsConnecting } from '$lib/stores/walletconnect';

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}  (click to disconnect)`
    : "CONNECT"
</script>

<div>
  <Alert color="yellow" border class="mb-8">
    <IconWarning slot="icon" />
    Only mobile wallets are supported in WalletConnect.
  </Alert>

  <div class="flex flex-col w-full justify-between space-y-2">
    <ButtonLoading
      color="blue"
      class="px-2 py-1"
      size="lg"
      pill
      loading={$walletconnectIsDisconnecting || $walletconnectIsConnecting}
      on:click={walletconnectConnect}
    >
    {walletconnectLabel}
    </ButtonLoading>
  </div>
</div>