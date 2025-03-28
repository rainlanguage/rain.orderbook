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

	export let token: GuiSelectTokensCfg;
	export let gui: DotrainOrderGui;
	export let onSelectTokenSelect: () => void;
	let inputValue: string | null = null;
	let tokenInfo: TokenInfo | null = null;
	let error = '';
	let checking = false;

	onMount(async () => {
		try {
			let result = await gui.getTokenInfo(token.key);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			tokenInfo = result.value;
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
			let result = await gui.getTokenInfo(token.key);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			tokenInfo = result.value;
			error = '';
		} catch {
			return (error = 'No token exists at this address.');
		}
	}

	async function handleInput(event: Event) {
		tokenInfo = null;
		const currentTarget = event.currentTarget;
		if (currentTarget instanceof HTMLInputElement) {
			inputValue = currentTarget.value;
			if (!inputValue) {
				error = '';
			}
			checking = true;
			try {
				await gui.saveSelectToken(token.key, currentTarget.value);
				await getInfoForSelectedToken();
			} catch (e) {
				const errorMessage = (e as Error).message ? (e as Error).message : 'Invalid token address.';
				error = errorMessage;
			}
		}

		checking = false;
		onSelectTokenSelect();
	}
</script>

<div class="flex w-full flex-col">
	<div class="flex flex-col gap-2">
		<div class="flex flex-col justify-start gap-4 lg:flex-row lg:items-center lg:justify-between">
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
		</div>
		<Input type="text" size="lg" on:input={handleInput} bind:value={inputValue} />
	</div>
</div>
