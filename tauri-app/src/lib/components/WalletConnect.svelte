<script lang="ts">
  import { Button } from 'flowbite-svelte';
  import SkeletonRow from './SkeletonRow.svelte';
  import { walletconnectModal, account } from '$lib/stores/settings';

  $: label = $account
    ? `${$account.slice(0, 5)}...${$account.slice(-1 * 5)}`
    : "CONNECT"

  function connect() {
    $walletconnectModal?.open(
      // {view: "Networks"}
    )
  }

  // import { Contract, ethers } from 'ethers'
    // import { toasts } from '$lib/stores/toasts';

  // // const USDTAddress = '0x617f3112bf5397D0467D315cC709EF968D9ba546'
  // const USDTAddress = "0x45c32fA6DF82ead1e2EF74d17b76547EDdFaFF89"

  // // The ERC-20 Contract ABI, which is a common contract interface
  // // for tokens (this is the Human-Readable ABI format)
  // const USDTAbi = [
  //   'function name() view returns (string)',
  //   'function symbol() view returns (string)',
  //   'function balanceOf(address) view returns (uint)',
  //   'function transfer(address to, uint amount)',
  //   'event Transfer(address indexed from, address indexed to, uint amount)'
  // ]


  // async function getBalance(calldata: Uint8Array, to: string, waiting?: number) {
  //   const walletProvider = $walletconnectModal?.getWalletProvider();
  //   if (!walletProvider || !$isConnected || !$account) toasts.error("user not connected");
  //   else {
  //     const ethersProvider = new ethers.providers.Web3Provider(walletProvider);
  //     const signer = ethersProvider.getSigner();
  //     const rawtx = {
  //       data: calldata,
  //       to,
  //     };
  //     const result = await signer.sendTransaction(rawtx);
  //     await result.wait(waiting);
  //     // const USDTContract = new Contract(USDTAddress, USDTAbi, signer)
  //     // const USDTBalance = await USDTContract.balanceOf("0x86aeF9BeaC77fee6f2165e515FcE6505F40afC27")
  //     // const result = await USDTContract.transfer("0xC903C23399e493cE46BcE302FAd48f9F4a6e5C6D", USDTBalance)

  //     // eslint-disable-next-line no-console
  //     // console.log(USDTBalance, result)
  //   }
  // }


</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <div class="absolute right-2 flex flex-col justify-center">
        {#if walletconnectModal === undefined}
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