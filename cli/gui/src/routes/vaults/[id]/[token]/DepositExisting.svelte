<script lang="ts">
	import type { TokenVaultsQuery } from '$lib/gql/generated/graphql';
	import type { Token } from '$lib';
	import type { VaultId } from '$lib';
	import Deposit from '../../Deposit.svelte';
	import { vaultQueryToToken } from '$lib/utils/vaults';

	export let vault: TokenVaultsQuery['tokenVaults'][0];

	let token: Token;
	let vaultId: VaultId;

	$: token = vaultQueryToToken(vault);
	$: vaultId = BigInt(vault.vaultId);
</script>

{#if token && vaultId}
	<Deposit {token} {vaultId} />
{/if}
