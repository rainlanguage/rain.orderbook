<script lang="ts">
  import { toasts } from "$lib/stores/toasts";
  import { Tooltip } from 'flowbite-svelte';
  import { WalletOutline, FingerprintOutline } from "flowbite-svelte-icons";
  import { HashType } from "$lib/utils/hash";

  export let value: string;
  export let type: HashType | undefined = undefined;
  export let chars = 6;

  $: id = `hash-${value}`;

  function copy() {
    navigator.clipboard.writeText(value);
    toasts.success("Copied to clipboard");
  }
</script>

<button type="button" {id} class="inline-block flex justify-start items-center space-x-2" on:click|stopPropagation={copy}>
  {#if type === HashType.Wallet }
    <WalletOutline size="sm" />
  {:else if type == HashType.Identifier }
    <FingerprintOutline size="sm" />
  {/if}
  <div>{value.slice(0, chars)}...{value.slice(-1 * chars)}</div>
</button>
<Tooltip triggeredBy={`#${id}`}>{value}</Tooltip>