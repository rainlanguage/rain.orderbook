<script lang="ts">
	import { transactionStore, type DeploymentArgs } from '@rainlanguage/ui-components';
	import { wagmiConfig } from '$lib/stores/wagmi';
	import TransactionModal from './TransactionModal.svelte';

	export let open: boolean = false;
	export let args: DeploymentArgs;

	const messages = {
		success: 'Your strategy was successfully deployed.',
		pending: 'Deploying your strategy...',
		error: 'Could not deploy strategy.'
	};

	transactionStore.handleDeploymentTransaction({
		config: $wagmiConfig,
		...args
	});
</script>

<TransactionModal bind:open {messages} />
