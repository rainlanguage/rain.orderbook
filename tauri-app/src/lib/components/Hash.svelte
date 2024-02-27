<script lang="ts">
  import { toasts } from "$lib/stores/toasts";
  import { Tooltip } from 'flowbite-svelte';
  import { WalletOutline, FingerprintOutline, ClipboardListOutline } from "flowbite-svelte-icons";
  import { HashType } from "$lib/types/hash";

  export let value: string;
  export let type: HashType | undefined = undefined;
  export let shorten = true;
  export let sliceLen = 5;
  export let copyOnClick = true;

  $: id = shorten ? `hash-${value}` : undefined;
  $: displayValue = value && shorten ? `${value.slice(0, sliceLen)}...${value.slice(-1 * sliceLen)}` : value;

  function copy(e) {
    if(copyOnClick) {
      e.stopPropagation();
      navigator.clipboard.writeText(value);
      toasts.success("Copied to clipboard");
    }
  }
</script>

<button type="button" {id} class="inline-block flex justify-start items-center space-x-2 text-left" on:click={copy}>
  {#if type === HashType.Wallet }
    <WalletOutline size="sm" />
  {:else if type === HashType.Identifier }
    <FingerprintOutline size="sm" />
  {:else if type === HashType.Transaction}
    <ClipboardListOutline size="sm" />
  {/if}
  <div>{displayValue}</div>
</button>

{#if shorten}
  <Tooltip triggeredBy={`#${id}`}>
    <div class="inline-block flex justify-start items-center space-x-2">
      {#if type === HashType.Wallet }
        <WalletOutline size="sm" />
      {:else if type === HashType.Identifier }
        <FingerprintOutline size="sm" />
      {:else if type === HashType.Transaction}
        <ClipboardListOutline size="sm" />
      {/if}
      <div>{value}</div>
    </div>
  </Tooltip>
{/if}