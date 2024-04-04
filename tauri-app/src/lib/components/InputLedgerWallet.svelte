<script lang="ts">
  import { Helper, Alert } from 'flowbite-svelte';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { toasts } from '$lib/stores/toasts';
  import { getAddressFromLedger } from '$lib/services/wallet';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import IconWarning from '$lib/components/IconWarning.svelte';
  import { ledgerWalletAddress, ledgerWalletDerivationIndex } from '$lib/stores/wallets';
  import ButtonLoading from './ButtonLoading.svelte';
  import Hash from './Hash.svelte';
  import { HashType } from '$lib/types/hash';

  const maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: 0,
    thousandsSeparator: '',
    radix: '.',
  };

  let derivationIndex: number = 0;
  let isConnecting: boolean;
  let isDisconnecting = false;

  function completeDerivationIndex({ detail }: { detail: InputMask }) {
    derivationIndex = parseInt(detail.unmaskedValue);
    ledgerWalletDerivationIndex.set(derivationIndex);
  }

  async function getAddress() {
    isConnecting = true;
    try {
      const res: string = await getAddressFromLedger(derivationIndex);
      ledgerWalletAddress.set(res)
    } catch (e) {
      reportErrorToSentry(e);
      toasts.error(`Ledger error: ${e as string}`);
    }
    isConnecting = false;
  }

  function ledgerDisconnect() {
    isDisconnecting = true;
    ledgerWalletAddress.set(undefined);
    ledgerWalletDerivationIndex.set(0);
    isDisconnecting = false;
  }
</script>

<div>
  <Alert color="yellow" border class="mb-8">
    <IconWarning slot="icon" />
    <div class="pl-2">
      <div class="mb-2 text-lg">Before you continue:</div>
      <ul role="list" class="list-disc pl-5 space-y-2">
        <li>All desktop applications linked to your Ledger wallet must be closed, including any desktop wallets and Ledger Live.</li>
        <li>Your Ledger wallet must be authenticated with the Ethereum app open.</li>
      </ul>
    </div>
  </Alert>

  <div class="flex w-full justify-end space-x-2 items-start">
    <ButtonLoading
      color="blue"
      class="px-2 py-1 w-full"
      size="lg"
      pill
      loading={isConnecting}
      on:click={getAddress}
    >
      {#if $ledgerWalletAddress}
        <Hash type={HashType.Wallet} value={$ledgerWalletAddress} />
      {:else}
        CONNECT
      {/if}
    </ButtonLoading>
    {#if $ledgerWalletAddress}
      <ButtonLoading
        color="red"
        class="px-2 py-1 min-w-fit"
        size="lg"
        pill
        loading={isDisconnecting}
        on:click={ledgerDisconnect}
      >
        DISCONNECT
      </ButtonLoading>
    {:else}
      <div class="w-32 grow-0 break-all">
        <input
          type="text"
          class="focus:border-primary-500 focus:ring-primary-500 dark:focus:border-primary-500 dark:focus:ring-primary-500 block w-32 rounded-lg border-gray-300 bg-gray-50 p-2.5 text-sm text-gray-900 disabled:cursor-not-allowed disabled:opacity-50 rtl:text-right dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
          value={derivationIndex}
          use:imask={maskOptions}
          on:complete={completeDerivationIndex}
        />
        <Helper class="break-word mt-2 text-sm">Derivation Index</Helper>
      </div>
    {/if}
  </div>
</div>