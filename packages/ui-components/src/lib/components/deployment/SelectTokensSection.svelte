<script lang="ts">
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import SelectToken from './SelectToken.svelte';
	import { DotrainOrderGui, type SelectTokens } from '@rainlanguage/orderbook/js_api';

	export let gui: DotrainOrderGui;
	export let selectTokens: SelectTokens;
	export let allTokensSelected: boolean;
</script>

<div class="flex w-full flex-col gap-4">
	<DeploymentSectionHeader
		title="Select Tokens"
		description="Select the tokens that you want to use in your order."
		open={true}
		value={undefined}
	/>
	{#each selectTokens as selectToken}
		<SelectToken bind:selectToken bind:gui bind:selectTokens bind:allTokensSelected />
	{/each}
</div>
