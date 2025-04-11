<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, GuiProvider, type DeploymentArgs } from '@rainlanguage/ui-components';
	import { connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDeployModal, handleDisclaimerModal } from '$lib/services/modal';
	import { DotrainOrderGui } from '@rainlanguage/orderbook';
	import { onMount } from 'svelte';
	import { handleGuiInitialization } from '$lib/services/handleGuiInitialization';
	import { REGISTRY_URL } from '$lib/constants';

	const { settings } = $page.data.stores;
	const { dotrain, deployment, strategyDetail } = $page.data;
	const stateFromUrl = $page.url.searchParams?.get('state') || '';

	let gui: DotrainOrderGui | null = null;
	let getGuiError: string | null = null;

	$: registryUrl = $page.url.searchParams?.get('registry') || REGISTRY_URL;

	if (!dotrain || !deployment) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}

	onMount(async () => {
		if (dotrain && deployment) {
			const { gui: initializedGui, error } = await handleGuiInitialization(
				dotrain,
				deployment.key,
				stateFromUrl
			);
			gui = initializedGui;
			getGuiError = error;
		}
	});

	const onDeploy = (deploymentArgs: DeploymentArgs) => {
		handleDisclaimerModal({
			open: true,
			onAccept: () => {
				handleDeployModal({
					args: deploymentArgs,
					open: true
				});
			}
		});
	};
</script>

{#if !dotrain || !deployment}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if gui}
	<GuiProvider {gui}>
		<DeploymentSteps
			{strategyDetail}
			{deployment}
			wagmiConnected={connected}
			{appKitModal}
			{onDeploy}
			{settings}
		/>
	</GuiProvider>
{:else if getGuiError}
	<div>
		{getGuiError}
	</div>
{/if}
