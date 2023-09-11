<script lang="ts">
	import { page } from '$app/stores';
	import { Icon } from 'flowbite-svelte-icons';
	import TokenVault from './TokenVault.svelte';
	import { onMount } from 'svelte';
	import { isHex } from 'viem';
	import { Button } from 'flowbite-svelte';

	let error: string;
	let vaultId: `0x${string}`;
	let token: `0x${string}`;

	onMount(() => {
		const { id: _vaultId, token: _token } = $page.params;
		try {
			if (!isHex(_vaultId)) {
				throw new Error('Invalid vault ID');
			}
			if (!isHex(_token)) {
				throw new Error('Invalid token address');
			}
			vaultId = _vaultId;
			token = _token;
		} catch {
			error = 'Invalid vault ID';
		}
	});
</script>

<div class="flex mb-4">
	<Button
		size="sm"
		outline
		on:click={() => {
			history.back();
		}}
	>
		<Icon name="angle-left-solid" class="inline-block mr-2 w-2" />
		<span>Back</span>
	</Button>
</div>

{#if vaultId}
	<TokenVault {vaultId} {token} />
{:else if error}
	<p>{error}</p>
{/if}
