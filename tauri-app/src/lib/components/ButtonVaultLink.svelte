<script lang="ts">
    import { goto } from "$app/navigation";
    import type { TokenVault } from "$lib/typeshare/orderDetail";
    import { bigintStringToHex } from "$lib/utils/hex";
    import { Button } from "flowbite-svelte";
    import Hash from "$lib/components/Hash.svelte";
    import { HashType } from "$lib/utils/hash";

    export let tokenVault: TokenVault;
</script>

<Button color="alternative" size="sm" on:click={() => goto(`/vaults/${tokenVault.id}`)} class="max-w-64 px-2">
  <div class="text-left w-full">
    <div class="text-lg mb-2 font-bold">{tokenVault.token.name}</div>
    <div class="mb-2 relative">
      <div class="font-bold">Vault ID</div>
      <div class="overflow-hidden overflow-ellipsis mb-1">{bigintStringToHex(tokenVault.vault_id)}</div>
    </div>
    <div class="mb-2">
      <div class="font-bold">Owner</div>
     <Hash type={HashType.Wallet} value={tokenVault.vault.owner.id} />
   </div>
  </div>
</Button>