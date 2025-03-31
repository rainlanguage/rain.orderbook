<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';
	import { createEventDispatcher } from 'svelte';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';
	import {
		getDeploymentTransactionArgs,
		type HandleAddOrderResult
	} from './getDeploymentTransactionArgs';
	import { useGui } from '$lib/hooks/useGui';
	import { useAccount } from '$lib/providers/wallet/useAccount';

	const dispatch = createEventDispatcher<{
		clickDeploy: {
			result: HandleAddOrderResult;
			networkKey: string;
			subgraphUrl: string;
		};
	}>();

	const gui = useGui();
	const { account } = useAccount();

	export let subgraphUrl: string;
	export let testId = 'deploy-button';

	let networkKey: string;
	let checkingDeployment = false;

	async function handleDeployButtonClick() {
		DeploymentStepsError.clear();

		let result: HandleAddOrderResult | null = null;
		checkingDeployment = true;

		try {
			const { value: networkKeyValue, error: networkKeyError } = gui.getNetworkKey();
			if (networkKeyError) {
				return DeploymentStepsError.catch(networkKeyError, DeploymentStepsErrorCode.NO_NETWORK_KEY);
			} else {
				networkKey = networkKeyValue;
			}
			result = await getDeploymentTransactionArgs(gui, $account);
			checkingDeployment = false;

			dispatch('clickDeploy', {
				result,
				networkKey,
				subgraphUrl
			});
		} catch (e) {
			checkingDeployment = false;
			return DeploymentStepsError.catch(e, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
		}
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
