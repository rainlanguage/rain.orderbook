<script lang="ts">
  import { Button, Input, Helper, Spinner } from 'flowbite-svelte';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { rpcUrl } from '$lib/stores/settings';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api';
  import { isAddress } from 'viem';
  import { toasts } from '$lib/stores/toasts';
  import { ToastMessageType } from '$lib/typeshare/toast';

  const maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: 0,
    thousandsSeparator: '',
    radix: '.',
  };

  let derivationIndexInput: number = 0;
  export let derivationIndex: number = 0;
  export let walletAddress: string;
  let isFetchingFromLedger: boolean;

  $: isWalletAddressValid = walletAddress && walletAddress.length > 0 && isAddress(walletAddress);

  function completeDerivationIndex({ detail }: { detail: InputMask }) {
    derivationIndex = parseInt(detail.unmaskedValue);
  }

  async function getAddressFromLedger() {
    isFetchingFromLedger = true;
    try {
      const res: string = await invoke('get_address_from_ledger', {
        derivationIndex,
        chainId: 137,
        rpcUrl: get(rpcUrl),
      });
      walletAddress = res;
    } catch (error) {
      toasts.add({
        message_type: ToastMessageType.Error,
        text: `Ledger error: ${error}`,
      });
    }
    isFetchingFromLedger = false;
  }
</script>

<div class="flex w-full items-start justify-start space-x-2">
  <div class="grow">
    <div class="relative flex">
      <Input
        label="Ledger Wallet Address"
        name="walletAddress"
        required
        bind:value={walletAddress}
      />
      <div class="absolute right-2 flex h-10 flex-col justify-center">
        <Button
          color="blue"
          class="px-2 py-1"
          size="xs"
          pill
          on:click={getAddressFromLedger}
          disabled={isFetchingFromLedger}
        >
          {#if isFetchingFromLedger}
            <Spinner size="3" class="mr-2" color="white" />
          {/if}
          CONNECT
        </Button>
      </div>
    </div>
    {#if !isWalletAddressValid && walletAddress.length > 0}
      <Helper class="mt-2 text-sm" color="red">Invalid Address</Helper>
    {/if}
    <Helper class="mt-2 text-sm">The address of your Ledger wallet.</Helper>
  </div>
  <div class="w-32 grow-0 break-all">
    <input
      class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-32 rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
      value={derivationIndexInput}
      use:imask={maskOptions}
      on:complete={completeDerivationIndex}
    />
    <Helper class="break-word mt-2 text-sm">Derivation Index</Helper>
  </div>
</div>
