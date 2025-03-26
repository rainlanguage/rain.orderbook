<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';
	import {
		getDeploymentTransactionArgs,
		type HandleAddOrderResult
	} from './getDeploymentTransactionArgs';
	import type { Writable } from 'svelte/store';
	import type { Config } from 'wagmi';
	import { useGui } from '$lib/hooks/useGui';

	const dispatch = createEventDispatcher<{
		clickDeploy: {
			result: HandleAddOrderResult;
			networkKey: string;
			subgraphUrl: string;
		};
	}>();

	const gui = useGui();
	const networkKey = gui.getNetworkKey();

	export let wagmiConfig: Writable<Config | undefined>;
	export let subgraphUrl: string;
	export let testId = 'deploy-button';

	let checkingDeployment = false;

	async function handleDeployButtonClick() {
		DeploymentStepsError.clear();

		let result: HandleAddOrderResult | null = null;
		checkingDeployment = true;

		try {
			result = await getDeploymentTransactionArgs(gui, $wagmiConfig);
		} catch (e) {
			checkingDeployment = false;
			return DeploymentStepsError.catch(e, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
		}

		checkingDeployment = false;

		dispatch('clickDeploy', {
			result,
			networkKey,
			subgraphUrl
		});
	}
</script>

<Button
	data-testid={testId}
	size="lg"
	disabled={checkingDeployment}
	on:click={handleDeployButtonClick}
	class="bg-gradient-to-br from-blue-600 to-violet-600"
>
	{#if checkingDeployment}
		<Spinner size="4" color="white" />
		<span class="ml-2">Checking deployment...</span>
	{:else}
		Deploy Strategy
	{/if}
</Button>
