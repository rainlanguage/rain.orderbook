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
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	// Create event dispatcher for custom events
	const dispatch = createEventDispatcher<{
		click: {
			type: 'showDisclaimer' | 'deploy';
			result: HandleAddOrderResult;
			networkKey: string;
			subgraphUrl: string;
			onSuccess?: () => void;
			[key: string]: any;
		};
	}>();

	export let wagmiConfig: Writable<Config | undefined>;
	export let gui: DotrainOrderGui;
	export let subgraphUrl: string;
	export let testId = 'deploy-button';
	export let disabled = false;

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

		if (!result) return;

		// Emit the acceptDisclaimer event
		dispatch('click', {
			type: 'showDisclaimer',
			result,
			networkKey,
			subgraphUrl,
			onSuccess: () => {
				// When disclaimer is accepted, emit the deploy event
				dispatch('click', {
					type: 'deploy',
					result,
					networkKey,
					subgraphUrl
				});
			}
		});
	}
</script>

<Button
	data-testid={testId}
	size="lg"
	{disabled}
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
