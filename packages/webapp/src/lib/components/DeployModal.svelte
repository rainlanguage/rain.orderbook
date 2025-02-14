<script lang="ts">
	import { transactionStore } from '@rainlanguage/ui-components';
	import type { Hex } from 'viem';
	import type {
		ApprovalCalldataResult,
		DepositAndAddOrderCalldataResult
	} from '@rainlanguage/orderbook/js_api';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import TransactionModal from './TransactionModal.svelte';

	export let open: boolean;
	export let approvals: ApprovalCalldataResult;
	export let deploymentCalldata: DepositAndAddOrderCalldataResult;
	export let orderbookAddress: Hex;
	export let chainId: number;
	export let subgraphUrl: string;

	const messages = {
		success: 'Your strategy was successfully deployed.',
		pending: 'Deploying your strategy...',
		error: 'Could not deploy strategy.'
	};

	transactionStore.handleDeploymentTransaction({
		config: $wagmiConfig,
		approvals,
		deploymentCalldata,
		orderbookAddress,
		chainId,
		subgraphUrl
	});
</script>

<TransactionModal bind:open {messages} />
