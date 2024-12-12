<script lang="ts">
  import { Helper, Alert } from 'flowbite-svelte';
  import type { InputMask } from 'imask';
  import { imask } from '@imask/svelte';
  import { toasts } from '$lib/stores/toasts';
  import { getAddressFromLedger } from '$lib/services/wallet';
  import { reportErrorToSentry } from '$lib/services/sentry';
  import { IconWarning, ButtonLoading } from '@rainlanguage/ui-components';
  import { ledgerWalletAddress, ledgerWalletDerivationIndex } from '$lib/stores/wallets';
  import { Hash, HashType } from '@rainlanguage/ui-components';

  const maskOptions = {
    mask: Number,
    min: 0,
    lazy: false,
    scale: 0,
    thousandsSeparator: '',
    radix: '.',
  };

  export let onConnect: () => void = () => {};
  let derivationIndex: number = 0;
  let isConnecting: boolean;
  let isDisconnecting = false;

  function completeDerivationIndex({ detail }: { detail: InputMask }) {
    derivationIndex = parseInt(detail.unmaskedValue);
    ledgerWalletDerivationIndex.set(derivationIndex);
  }

  async function ledgerConnect() {
    if (!$ledgerWalletAddress) {
      isConnecting = true;
      try {
        const res: string = await getAddressFromLedger(derivationIndex);
        ledgerWalletAddress.set(res);
        onConnect();
      } catch (e) {
        reportErrorToSentry(e);
        toasts.error(`Ledger error: ${e as string}`);
      }
      isConnecting = false;
    }
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
      <ul role="list" class="list-disc space-y-2 pl-5">
        <li>
          All desktop applications linked to your Ledger wallet must be closed, including any
          desktop wallets and Ledger Live.
        </li>
        <li>Your Ledger wallet must be authenticated with the Ethereum app open.</li>
      </ul>
    </div>
  </Alert>

  <div class="flex w-full items-start justify-end space-x-2">
    <ButtonLoading
      color="primary"
      class="w-full px-2 py-1"
      size="lg"
      pill
      loading={isConnecting}
      on:click={ledgerConnect}
    >
      {#if $ledgerWalletAddress}
        <Hash type={HashType.Wallet} value={$ledgerWalletAddress} />
      {:else}
        Connect
      {/if}
    </ButtonLoading>
    {#if $ledgerWalletAddress}
      <ButtonLoading
        color="red"
        class="min-w-fit px-2 py-1"
        size="lg"
        pill
        loading={isDisconnecting}
        on:click={ledgerDisconnect}
      >
        Disconnect
      </ButtonLoading>
    {:else}
      <div class="w-32 grow-0 break-all">
        <input
          type="text"
          class="block w-32 rounded-xl border-gray-300 bg-gray-50 p-1.5 text-sm text-gray-900 focus:border-primary-500 focus:ring-primary-500 disabled:cursor-not-allowed disabled:opacity-50 dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400 dark:focus:border-primary-500 dark:focus:ring-primary-500 rtl:text-right"
          value={derivationIndex}
          use:imask={maskOptions}
          on:complete={completeDerivationIndex}
        />
        <Helper class="break-word mt-2 text-sm">Derivation Index</Helper>
      </div>
    {/if}
  </div>
</div>
