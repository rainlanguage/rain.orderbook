<script lang="ts">
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import SelectToken from './SelectToken.svelte';
	import { DotrainOrderGui, type GuiSelectTokensCfg } from '@rainlanguage/orderbook/js_api';

	export let gui: DotrainOrderGui;
	export let selectTokens: GuiSelectTokensCfg[];
	export let onSelectTokenSelect: () => void;
</script>

<div class="flex w-full flex-col gap-4">
	<DeploymentSectionHeader
		title="Select Tokens"
		description="Select the tokens that you want to use in your order."
	/>
	{#each selectTokens as token}
		<SelectToken {token} bind:gui {onSelectTokenSelect} />
	{/each}
</div>
