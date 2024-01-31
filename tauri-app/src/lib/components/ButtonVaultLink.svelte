<script lang="ts">
    import { goto } from "$app/navigation";
    import type { TokenVault } from "$lib/typeshare/orderDetail";
    import { formatAddressShorthand } from "$lib/utils/address";
    import { Button } from "flowbite-svelte";
    import { toHex } from "viem";

    export let tokenVault: TokenVault;
</script>
<Button color="alternative" size="sm" on:click={() => goto(`/vaults/${tokenVault.id}`)} class="max-w-64 px-2">
  <div class="text-left w-full">
    <div class="text-lg mb-2 font-bold">{tokenVault.token.name}</div>
    <div class="mb-1 overflow-hidden text-ellipsis"><span class="font-bold">Vault ID</span> {toHex(tokenVault.vault_id)}</div>
    <div><span class="mb-1 font-bold">Owner</span> {formatAddressShorthand(tokenVault.vault.owner.id)}</div>
  </div>
</Button>