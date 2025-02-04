<script lang="ts">
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import SelectToken from './SelectToken.svelte';
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let gui: DotrainOrderGui;
	export let selectTokens: string[];
	export let allTokensSelected: boolean;
</script>

<div class="flex w-full flex-col gap-4">
	<DeploymentSectionHeader
		title="Select Tokens"
		description="Select the tokens that you want to use in your order."
		open={true}
		value={undefined}
	/>
	{#each selectTokens as tokenKey}
		<SelectToken {tokenKey} bind:gui bind:selectTokens bind:allTokensSelected />
	{/each}
</div>
