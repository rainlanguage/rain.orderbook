<script lang="ts">
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import SelectToken from './SelectToken.svelte';
	import { DotrainOrderGui, type SelectTokens } from '@rainlanguage/orderbook/js_api';

	export let gui: DotrainOrderGui;
	export let selectTokens: SelectTokens;
	export let allTokensSelected: boolean;
	export let handleUpdateGuiState: (gui: DotrainOrderGui) => void;
</script>

<div class="flex w-full flex-col gap-4">
	<DeploymentSectionHeader
		title="Select Tokens"
		description="Select the tokens that you want to use in your order."
	/>
	{#each selectTokens as tokenKey}
		<SelectToken
			{tokenKey}
			bind:gui
			bind:selectTokens
			bind:allTokensSelected
			{handleUpdateGuiState}
		/>
	{/each}
</div>
