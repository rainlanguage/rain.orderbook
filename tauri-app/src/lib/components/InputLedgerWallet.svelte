<script lang="ts">
  import { Alert } from 'flowbite-svelte';
  import IconWarning from '$lib/components/IconWarning.svelte';
  import InputLedgerDerivationAddressSelect from '$lib/components/InputLedgerDerivationAddressSelect.svelte';
  import InputLedgerDerivationAddressManual from '$lib/components/InputLedgerDerivationAddressManual.svelte';

  export let derivationIndex: number;
  export let walletAddress: string;
</script>

<div class="relative">
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

  <div class="flex flex-col space-y-4">
    <InputLedgerDerivationAddressSelect bind:index={derivationIndex} bind:address={walletAddress} />
    <InputLedgerDerivationAddressManual bind:index={derivationIndex} bind:address={walletAddress} />
  </div>
</div>
