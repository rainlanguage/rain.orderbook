<script lang="ts">
  import { Button } from 'flowbite-svelte';
  import { disconnect, getAccount } from '@wagmi/core';
  import { wagmiConfig } from '$lib/stores/settings';
  import { createWeb3Modal } from '@web3modal/wagmi';
  import SkeletonRow from './SkeletonRow.svelte';

  let modal: ReturnType<typeof createWeb3Modal>;
  let a: ReturnType<typeof modal.subscribeEvents>;
  let b: ReturnType<typeof modal.subscribeState>;
  $: if ($wagmiConfig) {
    if (a) a();
    if (b) b();
    console.log(getAccount($wagmiConfig))
    modal = createWeb3Modal({
      wagmiConfig: $wagmiConfig,
      projectId: "634cfe0b2781e2ac78219ca4cb23c13f",
      enableAnalytics: true, // Optional - defaults to your Cloud configuration
      // enableOnramp: true, // Optional - false as default
      allWallets: "HIDE",
      includeWalletIds: [
        "e7c4d26541a7fd84dbdfa9922d3ad21e936e13a7a0e44385d44f006139e44d3b" // walletconnect
      ],
      defaultChain: $wagmiConfig.chains[0]
    })
    a = modal.subscribeEvents(v => {
      console.log(v)
      if (v.data.event === "MODAL_CLOSE" && v.data.properties.connected) {
        account = $wagmiConfig ? getAccount($wagmiConfig) : undefined;
      }
      if (v.data.event === "DISCONNECT_SUCCESS") {
        disconnect($wagmiConfig)
      }
    })
    b = modal.subscribeState(v => {
      console.log(v)
    })
  }

  $: account = $wagmiConfig ? getAccount($wagmiConfig) : undefined;

  $: label = account && account.isConnected
    ? account.address
      ? `${account.address.slice(0, 5)}...${account.address.slice(-1 * 5)}`
      : "DISCONNECT"
    : "CONNECT"

  function connect() {
    modal.open(
        // {view: "Networks"}
      )
    // // console.log("yo");
    // if (getAccount($wagmiConfig)?.isConnected) {
    //   console.log(getAccount($wagmiConfig))
    //   console.log("yo1");
    //   disconnect($wagmiConfig!, { connector: account!.connector})
    //   // modal.open(
    //   //   // {view: "Networks"}
    //   // )
    // } else {
    //   console.log("yo2");
    //   console.log(getAccount($wagmiConfig))
    //   console.log($wagmiConfig.chains)
    //   modal.open(
    //     // {view: "Networks"}
    //   )
    // }
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <div class="absolute right-2 flex flex-col justify-center">
        {#if modal === undefined}
          <SkeletonRow />
        {:else}
          <Button
            color="blue"
            class="px-2 py-1"
            size="xs"
            pill
            on:click={connect}
          >
          {label}
          </Button>
        {/if}
      </div>
    </div>
  </div>
</div>
