<script lang="ts">
  import { Button, Modal } from 'flowbite-svelte';
  import InputWalletConnect from '$lib/components/InputWalletConnect.svelte';
  import { IconWalletConnect } from '@rainlanguage/ui-components';
  import { walletconnectAccount } from '$lib/stores/walletconnect';

  let open = false;

  $: label = $walletconnectAccount
    ? `${$walletconnectAccount.slice(0, 5)}...${$walletconnectAccount.slice(-5)}`
    : 'Connect to Wallet';

  function reset() {
    open = false;
  }
</script>

<div class="flex w-full flex-col py-4">
  <Button color="primary" pill on:click={() => (open = true)}>{label}</Button>
</div>

<Modal title="Connect to Wallet" bind:open outsideclose size="sm" on:close={reset}>
  <div class="flex justify-center">
    <Button class="text-lg" on:click={() => (open = true)}>
      <div class="mr-3">
        <IconWalletConnect />
      </div>
      WalletConnect
    </Button>
  </div>
  <InputWalletConnect onConnect={reset} />
</Modal>
