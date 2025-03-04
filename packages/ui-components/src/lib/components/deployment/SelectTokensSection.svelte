<script lang="ts">
	import DeploymentSectionHeader from './DeploymentSectionHeader.svelte';
	import SelectToken from './SelectToken.svelte';
	import { DotrainOrderGui, type SelectTokens } from '@rainlanguage/orderbook/js_api';
	import type { ExtendedTokenInfo } from '../../types/tokens';
	import { getViemChain } from '../../services/getViemChain';

	export let gui: DotrainOrderGui;
	export let selectTokens: SelectTokens;
	export let onSelectTokenSelect: () => void;
	export let tokenList: ExtendedTokenInfo[];
	export let networkKey: string;
	const chainId = getViemChain(networkKey).id;
	$: filteredTokenList = tokenList.filter((token) => token.chainId === chainId);
</script>

<div class="flex w-full flex-col gap-4">
	<DeploymentSectionHeader
		title="Select Tokens"
		description="Select the tokens that you want to use in your order."
	/>
	{#each selectTokens as token}
		<SelectToken {token} bind:gui {onSelectTokenSelect} tokenList={filteredTokenList} />
	{/each}
</div>
