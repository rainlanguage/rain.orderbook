<script lang="ts">
	import { Label, Input } from 'flowbite-svelte';
	import type { DotrainOrderGui, TokenInfo } from '@rainlanguage/orderbook/js_api';
	import { CheckCircleSolid, CloseCircleSolid } from 'flowbite-svelte-icons';
	import { Spinner } from 'flowbite-svelte';

	export let tokenKey: string;
	export let gui: DotrainOrderGui;
	export let selectTokens: string[];
	export let allTokensSelected: boolean;

	let inputValue: string | null = null;
	let tokenInfo: TokenInfo | null = null;
	let error = '';
	let checking = false;

	function checkIfAllTokensAreSelected() {
		if (selectTokens?.every((t) => gui?.isSelectTokenSet(t))) {
			allTokensSelected = true;
			console.log('all tokens are selected');
		} else {
			console.log('TOKENS NOT SELECTED');
			allTokensSelected = false;
		}
	}

	async function getInfoForSelectedToken() {
		error = '';
		try {
			tokenInfo = await gui.getTokenInfo(tokenKey);
			error = '';
		} catch (e) {
			console.error(e);
			return (error = 'No token exists at this address.');
		}
	}

	async function handleInput(event: Event) {
		console.log('INPUT');
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
					console.log('Replacing token');
					await gui.replaceSelectToken(tokenKey, currentTarget.value);
				} else {
					await gui.saveSelectToken(tokenKey, currentTarget.value);
				}
				await getInfoForSelectedToken();
			} catch (e) {
				console.error(e);
				error = 'Invalid token address.';
			}
		}
		checking = false;
		checkIfAllTokensAreSelected();
	}
</script>

<div class="mb-4 flex w-full max-w-2xl flex-col">
	<div class="flex flex-col gap-4">
		<div class="flex flex-row items-center gap-6">
			<Label class="whitespace-nowrap text-xl">{tokenKey}</Label>
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
				<div class="flex h-5 flex-row items-center gap-2">
					<CloseCircleSolid class="h-5 w-5" color="red" />
					<span>{error}</span>
				</div>
			{/if}
		</div>
		<Input type="text" size="lg" on:input={handleInput} bind:value={inputValue} />
	</div>
</div>
