<script lang="ts">
  import { Button } from 'flowbite-svelte';
  import { config, modal } from '$lib/services/walletConnect';
  import { disconnect, getAccount } from '@wagmi/core'

  function connect() {
    if (getAccount(config).isConnected) {
      disconnect(config)
    } else {
      // eslint-disable-next-line no-console
      console.log("yo");
      modal.open({view: "Networks"})
    }
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <div class="absolute right-2 flex h-10 flex-col justify-center">
        <Button
          color="blue"
          class="px-2 py-1"
          size="xs"
          pill
          on:click={connect}
        >
          CONNECT
        </Button>
      </div>
    </div>
  </div>
</div>
