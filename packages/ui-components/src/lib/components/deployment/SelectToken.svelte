<script lang="ts">
	import { Input } from 'flowbite-svelte';
	import type {
		DotrainOrderGui,
		GuiSelectTokensCfg,
		TokenInfo
	} from '@rainlanguage/orderbook/js_api';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import { Spinner } from 'flowbite-svelte';
	import { onMount } from 'svelte';
	import type { ExtendedTokenInfo } from '../../types/tokens';
	import TokenSearchBox from './TokenSearchBox.svelte';

	export let token: GuiSelectTokensCfg;
	export let gui: DotrainOrderGui;
	export let onSelectTokenSelect: () => void;
	export let tokenList: ExtendedTokenInfo[];

	let inputValue: string | null = null;
	let tokenInfo: TokenInfo | null = null;
	let error = '';
	let checking = false;

	onMount(async () => {
		try {
			tokenInfo = await gui?.getTokenInfo(token.key);
			if (tokenInfo?.address) {
				inputValue = tokenInfo.address;
			}
		} catch {
			// do nothing
		}
	});

	async function getInfoForSelectedToken() {
		error = '';
		try {
			tokenInfo = await gui.getTokenInfo(token.key);
			error = '';
		} catch {
			return (error = 'No token exists at this address.');
		}
	}

	async function handleTokenUpdate(address: string) {
		tokenInfo = null;
		inputValue = address;
		if (!inputValue) {
			error = '';
			return;
		}
		checking = true;
		try {
			if (gui.isSelectTokenSet(token.key)) {
				await gui.replaceSelectToken(token.key, address);
			} else {
				await gui.saveSelectToken(token.key, address);
			}
			await getInfoForSelectedToken();
		} catch (e) {
			const errorMessage = (e as Error).message ? (e as Error).message : 'Invalid token address.';
			error = errorMessage;
		}
		checking = false;
		onSelectTokenSelect();
	}

	async function handleInput(event: Event) {
		const currentTarget = event.currentTarget;
		if (currentTarget instanceof HTMLInputElement) {
			await handleTokenUpdate(currentTarget.value);
		}
	}

	async function handleSelect(e: ExtendedTokenInfo) {
		await handleTokenUpdate(e.address);
	}
</script>

<div class="flex w-full flex-col">
	<div class="flex flex-col gap-2">
		<div class="flex flex-col justify-start gap-4">
			{#if token.name || token.description}
				<div class="flex flex-col">
					{#if token.name}
						<h1 class="break-words text-xl font-semibold text-gray-900 lg:text-xl dark:text-white">
							{token.name}
						</h1>
					{/if}
					{#if token.description}
						<p class="text-sm font-light text-gray-600 lg:text-base dark:text-gray-400">
							{token.description}
						</p>
					{/if}
				</div>
			{/if}
			{#if checking}
				<div class="flex h-5 flex-row items-center gap-2">
					<Spinner class="h-5 w-5" />
					<span>Checking...</span>
				</div>
			{:else if tokenInfo}
				<div class="flex h-5 flex-row items-center gap-2">
					<CheckCircleSolid class="h-5 w-5" color="green" />
					<span>{tokenInfo.name}</span>
				</div>
			{:else if error}
				<div class="flex h-5 flex-row items-center gap-2" data-testid="error">
					<CloseCircleSolid class="h-5 w-5" color="red" />
					<span>{error}</span>
				</div>
			{/if}
			<Input
				placeholder="Enter a custom token address"
				type="text"
				size="lg"
				on:input={handleInput}
				bind:value={inputValue}
			/>
			{#if tokenList.length > 0}
				<TokenSearchBox {tokenList} on:select={(e) => handleSelect(e.detail)} />
			{/if}
		</div>
	</div>
</div>
