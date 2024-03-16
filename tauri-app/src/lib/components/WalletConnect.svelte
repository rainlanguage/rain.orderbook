<script lang="ts">
  import { Button } from 'flowbite-svelte';
  import { walletconnectModal, account } from '$lib/stores/settings';

  $: label = $account
    ? `${$account.slice(0, 5)}...${$account.slice(-5)}`
    : "CONNECT"

  function connect() {
    $walletconnectModal?.open(
      // {view: "Networks"}
    )
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <div class="absolute right-2 flex flex-col justify-center">
        <Button
            color="blue"
            class="px-2 py-1"
            size="xs"
            pill
            disabled={$walletconnectModal === undefined}
            on:click={connect}
          >
          {label}
          </Button>
      </div>
    </div>
  </div>
</div>