<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';
	import {
		getDeploymentTransactionArgs,
		type HandleAddOrderResult
	} from './getDeploymentTransactionArgs';

	import type { DeployModalProps, DisclaimerModalProps } from '$lib/types/modal';
	import type { Writable } from 'svelte/store';
	import type { Config } from 'wagmi';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let handleDeployModal: (args: DeployModalProps) => void;
	export let handleDisclaimerModal: (args: DisclaimerModalProps) => void;
	export let wagmiConfig: Writable<Config | undefined>;
	export let gui: DotrainOrderGui;
	export let subgraphUrl: string;

	let checkingDeployment = false;
	const networkKey = gui.getNetworkKey();

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
		const onAccept = () => {
			handleDeployModal({
				open: true,
				args: {
					...result,
					subgraphUrl,
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
