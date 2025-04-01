<script lang="ts">
	import ButtonVaultLink from '../lib/components/ButtonVaultLink.svelte';
	import DepositOrWithdrawButtons from '../lib/components/detail/DepositOrWithdrawButtons.svelte';
	import type { SgVault } from '@rainlanguage/orderbook/js_api';
	import { isAddress, isAddressEqual } from 'viem';

	export let tokenVault: SgVault;
	export let subgraphName: string;
	export let handleDepositOrWithdrawModal: any = undefined;
	export let signerAddress: string = '';
	export let chainId: number | undefined = undefined;
	export let rpcUrl: string = '';
	export let orderDetailQuery: any = undefined;
	export let subgraphUrl: string = '';
</script>

<ButtonVaultLink {tokenVault} {subgraphName}>
	<svelte:fragment slot="buttons">
		{#if handleDepositOrWithdrawModal && signerAddress && isAddress(signerAddress) && isAddress(tokenVault.owner) && isAddressEqual(signerAddress, tokenVault.owner) && chainId}
			<div data-testid="buttons-rendered">
				<DepositOrWithdrawButtons
					vault={tokenVault}
					{chainId}
					{rpcUrl}
					query={orderDetailQuery}
					{handleDepositOrWithdrawModal}
					{subgraphUrl}
				/>
			</div>
		{/if}
	</svelte:fragment>
</ButtonVaultLink>
