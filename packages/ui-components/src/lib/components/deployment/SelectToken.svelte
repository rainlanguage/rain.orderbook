<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import type { DotrainOrderGui, TokenInfo } from '@rainlanguage/orderbook/js_api';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import { Spinner } from 'flowbite-svelte';
	import { onMount } from 'svelte';

	export let tokenKey: string;
	export let gui: DotrainOrderGui;
	export let selectTokens: string[];
	export let allTokensSelected: boolean;

	let inputValue: string | null = null;
	let tokenInfo: TokenInfo | null = null;
	let error = '';
	let checking = false;

	onMount(async () => {
		try {
			const currentToken = await gui?.getTokenInfo(tokenKey);
			if (currentToken?.address) {
				inputValue = currentToken.address;
				getInfoForSelectedToken();
			}
		} catch {
			// do nothing
		}
	});

	function checkIfAllTokensAreSelected() {
		allTokensSelected = false;
		if (selectTokens?.every((t) => gui?.isSelectTokenSet(t))) {
			allTokensSelected = true;
		} else {
			allTokensSelected = false;
		}
	}

	async function getInfoForSelectedToken() {
		error = '';
		try {
			tokenInfo = await gui.getTokenInfo(tokenKey);
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
			if (!gui) return;
			if (!inputValue) {
				error = '';
			}
			checking = true;
			try {
				if (gui.isSelectTokenSet(tokenKey)) {
					await gui.replaceSelectToken(tokenKey, currentTarget.value);
				} else {
					await gui.saveSelectToken(tokenKey, currentTarget.value);
				}
				await getInfoForSelectedToken();
			} catch (e) {
				const errorMessage = (e as Error).message ? (e as Error).message : 'Invalid token address.';
				error = errorMessage;
			}
		}
		checkIfAllTokensAreSelected();
		checking = false;
	}
</script>

<div class="flex w-full flex-col">
	<div class="flex flex-col gap-2">
		<div class="flex flex-row items-center gap-4">
			<Label class="whitespace-nowrap text-lg">{tokenKey}</Label>
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
