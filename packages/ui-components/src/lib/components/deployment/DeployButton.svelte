<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';

	import {
		getDeploymentTransactionArgs,
		type HandleAddOrderResult
	} from './getDeploymentTransactionArgs';
	import { useGui } from '../../hooks/useGui';

	import { wagmiConfig } from '../../stores/wagmi';
	let checkingDeployment = false;
	let gui = useGui();

	async function handleDeployButtonClick() {
		DeploymentStepsError.clear();

		if (!allTokenOutputs) {
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_TOKEN_OUTPUTS);
			return;
		}

		// TODO: remove this once we have a way to get the network key
		if (!networkKey) {
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_CHAIN);
			return;
		}

		let result: HandleAddOrderResult | null = null;
		checkingDeployment = true;
		try {
			result = await getDeploymentTransactionArgs(gui, $wagmiConfig);
		} catch (e) {
			checkingDeployment = false;
			DeploymentStepsError.catch(e, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
		}
		if (!result) {
			checkingDeployment = false;
			DeploymentStepsError.catch(null, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
			return;
		}
		checkingDeployment = false;
		const onAccept = () => {
			if (!networkKey) {
				DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_CHAIN);
				return;
			}

			handleDeployModal({
				open: true,
				args: {
					...result,
					subgraphUrl: subgraphUrl,
					network: networkKey
				}
			});
		};

		handleDisclaimerModal({
			open: true,
			onAccept
		});
	}
</script>

<Button
	size="lg"
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
