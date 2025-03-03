<script lang="ts" context="module">
	export enum HashType {
		Identifier,
		Wallet,
		Transaction,
		Address
	}
</script>

<script lang="ts">
	import { getExplorerLink } from '$lib/services/getExplorerLink';
	import truncateEthAddress from 'truncate-eth-address';
	import { Tooltip } from 'flowbite-svelte';
	import {
		WalletOutline,
		FingerprintOutline,
		ClipboardListOutline,
		ClipboardOutline
	} from 'flowbite-svelte-icons';
	import { fade } from 'svelte/transition';

	export let value: string;
	export let type: HashType | undefined = undefined;
	export let shorten = true;
	export let copyOnClick = true;
	let externalLink: boolean = false;
	export let chainId: number | undefined = undefined;
	export let linkType: 'tx' | 'address' | undefined = undefined;
	let showCopiedMessage = false;
	let explorerLink = '';

	$: if (chainId && linkType) {
		externalLink = true;
		explorerLink = getExplorerLink(value, chainId, linkType);
	}

	let cursorX = 0;
	let cursorY = 0;

	$: id = shorten ? `hash-${value}` : undefined;
	$: displayValue = value && shorten ? truncateEthAddress(value) : value;

	function copy(e: MouseEvent) {
		if (copyOnClick) {
			e.stopPropagation();
			navigator.clipboard.writeText(value);
			cursorX = e.clientX;
			cursorY = e.clientY;
			showCopiedMessage = true;
			setTimeout(() => {
				showCopiedMessage = false;
			}, 1500);
		}
	}
</script>

{#if externalLink}
	<a
		data-testid="external-link"
		href={explorerLink}
		target="_blank"
		rel="noopener noreferrer"
		{id}
		class="flex items-center justify-start space-x-2 text-left"
	>
		{#if type === HashType.Wallet}
			<WalletOutline size="sm" />
		{:else if type === HashType.Identifier}
			<FingerprintOutline size="sm" />
		{:else if type === HashType.Transaction}
			<ClipboardListOutline size="sm" />
		{:else if type === HashType.Address}
			<ClipboardOutline size="sm" />
		{/if}
		<div class="cursor-pointer hover:underline">{displayValue}</div>
	</a>
{:else}
	<button
		type="button"
		{id}
		class="flex items-center justify-start space-x-2 text-left"
		on:click={copy}
	>
		{#if type === HashType.Wallet}
			<WalletOutline size="sm" />
		{:else if type === HashType.Identifier}
			<FingerprintOutline size="sm" />
		{:else if type === HashType.Transaction}
			<ClipboardListOutline size="sm" />
		{:else if type === HashType.Address}
			<ClipboardOutline size="sm" />
		{/if}
		<div>{displayValue}</div>
	</button>
{/if}

{#if showCopiedMessage}
	<div
		out:fade
		class="fixed rounded bg-green-500 px-2 py-1 text-xs text-white shadow"
		style="top: {cursorY + 10}px; left: {cursorX + 10}px"
	>
		Copied to clipboard
	</div>
{/if}
