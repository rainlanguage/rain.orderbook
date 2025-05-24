<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, GuiProvider } from '@rainlanguage/ui-components';
	import { connected, appKitModal } from '$lib/stores/wagmi';
	import { DotrainOrderGui } from '@rainlanguage/orderbook';
	import { onMount } from 'svelte';
	import { handleGuiInitialization } from '$lib/services/handleGuiInitialization';
	import { useAccount } from '@rainlanguage/ui-components';
	import { handleDeploy } from '$lib/services/handleDeploy';

	const { settings } = $page.data.stores;
	const { dotrain, deployment, strategyDetail } = $page.data;
	const stateFromUrl = $page.url.searchParams?.get('state') || '';

	let gui: DotrainOrderGui | null = null;
	let getGuiError: string | null = null;

	const { account } = useAccount();

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
</script>

{#if !dotrain || !deployment}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if gui}
	<div data-testid="gui-provider">
		<GuiProvider {gui}>
			<DeploymentSteps
				{strategyDetail}
				{deployment}
				wagmiConnected={connected}
				{appKitModal}
				onDeploy={handleDeploy}
				{settings}
				{account}
			/>
		</GuiProvider>
	</div>
{:else if getGuiError}
	<div>
		{getGuiError}
	</div>
{/if}
