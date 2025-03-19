<script lang="ts">
	import { Button, Spinner } from 'flowbite-svelte';
	import { DeploymentStepsError, DeploymentStepsErrorCode } from '$lib/errors';
	import {
		getDeploymentTransactionArgs,
		type HandleAddOrderResult
	} from './getDeploymentTransactionArgs';
	import { useGui } from '../../hooks/useGui';
	import type { DeployModalProps, DisclaimerModalProps } from '$lib/types/modal';
	import type { Writable } from 'svelte/store';
	import type { Config } from 'wagmi';
	import { OrderbookYaml } from '@rainlanguage/orderbook/js_api';
	export let handleDeployModal: (args: DeployModalProps) => void;
	export let handleDisclaimerModal: (args: DisclaimerModalProps) => void;
	export let wagmiConfig: Writable<Config>;

	let checkingDeployment = false;
	const gui = useGui();

	const orderbookYaml = new OrderbookYaml([gui.generateDotrainText()]);
	const deployment = gui.getCurrentDeployment();
	const orderbookAddress = deployment.deployment.order?.orderbook?.address as string;
	const orderbook = orderbookYaml.getOrderbookByAddress(orderbookAddress);

	async function handleDeployButtonClick() {
		DeploymentStepsError.clear();

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
			handleDeployModal({
				open: true,
				args: {
					...result,
					subgraphUrl: orderbook.subgraph.url,
					chainId: orderbook.network.chainId,
					orderbookAddress: orderbook.address
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
