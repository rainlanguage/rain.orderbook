<script lang="ts">
  import IconWarning from "$lib/components/IconWarning.svelte";
  import { walletconnectModal, walletconnectAccount } from '$lib/stores/walletconnect';
  import { Alert, Button } from "flowbite-svelte";

  $: walletconnectLabel = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : "CONNECT"
</script>

<div>
  <Alert color="yellow" border class="mb-8">
    <IconWarning slot="icon" />
    Only mobile wallets are supported in WalletConnect.
  </Alert>

  <div class="flex flex-col w-full justify-between space-y-2">
    <Button
      color="blue"
      class="px-2 py-1"
      size="lg"
      pill
      on:click={() => $walletconnectModal?.open()}
    >
    {walletconnectLabel}
    </Button>
  </div>
</div>