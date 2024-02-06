<script lang="ts">
  import { toasts } from "$lib/stores/toasts";
  import { Tooltip } from 'flowbite-svelte';
  import { WalletOutline, FingerprintOutline, ClipboardListOutline } from "flowbite-svelte-icons";
  import { HashType } from "$lib/utils/hash";

  export let value: string;
  export let type: HashType | undefined = undefined;
  export let short = true;
  export let sliceLen = 5;

  $: id = `hash-${value}`;
  $: displayValue = value && short ? `${value.slice(0, sliceLen)}...${value.slice(-1 * sliceLen)}` : value;

  function copy() {
    navigator.clipboard.writeText(value);
    toasts.success("Copied to clipboard");
  }
</script>

<button type="button" {id} class="inline-block flex justify-start items-center space-x-2" on:click|stopPropagation={copy}>
  {#if type === HashType.Wallet }
    <WalletOutline size="sm" />
  {:else if type === HashType.Identifier }
    <FingerprintOutline size="sm" />
  {:else if type === HashType.Transaction}
    <ClipboardListOutline size="sm" />
  {/if}
  <div>{displayValue}</div>
</button>

{#if short}
  <Tooltip triggeredBy={`#${id}`}>{value}</Tooltip>
{/if}