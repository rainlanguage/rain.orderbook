<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import {
		DeploymentSteps,
		GuiProvider,
		useAccount,
		useToasts,
		useTransactions
	} from '@rainlanguage/ui-components';
	import { connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDisclaimerModal } from '$lib/services/modal';
	import { DotrainOrderGui, RaindexClient } from '@rainlanguage/orderbook';
	import { onMount } from 'svelte';
	import { handleGuiInitialization } from '$lib/services/handleGuiInitialization';
	import { handleAddOrder } from '$lib/services/handleAddOrder';
	import { handleTransactionConfirmationModal } from '$lib/services/modal';

	const { dotrain, deployment, orderDetail } = $page.data;
	const stateFromUrl = $page.url.searchParams?.get('state') || '';

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();

	let gui: DotrainOrderGui | null = null;
	let getGuiError: string | null = null;

	if (!dotrain || !deployment) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}

	onMount(async () => {
		if (dotrain && deployment) {
			const { gui: initializedGui, error } = await handleGuiInitialization(
				dotrain,
				undefined,
				deployment.key,
				stateFromUrl
			);
			gui = initializedGui;
			getGuiError = error;
		}
	});

	const onDeploy = (raindexClient: RaindexClient, gui: DotrainOrderGui) => {
		handleDisclaimerModal({
			open: true,
			onAccept: () => {
				handleAddOrder({
					raindexClient,
					handleTransactionConfirmationModal,
					errToast,
					manager,
					gui,
					account: $account
				});
			}
		});
	};
</script>

{#if !dotrain || !deployment}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if gui}
	<div data-testid="gui-provider">
		<GuiProvider {gui}>
			<DeploymentSteps
				{orderDetail}
				{deployment}
				wagmiConnected={connected}
				{appKitModal}
				{onDeploy}
				{account}
			/>
		</GuiProvider>
	</div>
{:else if getGuiError}
	<div>
		{getGuiError}
	</div>
{/if}
