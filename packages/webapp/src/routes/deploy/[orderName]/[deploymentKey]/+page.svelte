<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import {
		DeploymentSteps,
		GuiProvider,
		useAccount,
		useToasts,
		useTransactions,
		useRegistry
	} from '@rainlanguage/ui-components';
	import { connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDisclaimerModal } from '$lib/services/modal';
	import {
		DotrainOrderGui,
		RaindexClient,
		type NameAndDescriptionCfg
	} from '@rainlanguage/orderbook';
	import { handleAddOrder } from '$lib/services/handleAddOrder';
	import { handleTransactionConfirmationModal } from '$lib/services/modal';
	import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

	const { orderName } = $page.params as { orderName: string };
	const { deploymentKey } = $page.params as { deploymentKey: string };
	const stateFromUrl = $page.url.searchParams?.get('state');

	const { account } = useAccount();
	const { manager } = useTransactions();
	const { errToast } = useToasts();
	const { registry, loading, error } = useRegistry();

	let gui: DotrainOrderGui | null = null;
	let getGuiError: string | null = null;
	let deployment: { key: string; name: string; description: string } | null = null;
	let orderDetail: NameAndDescriptionCfg | null = null;

	if (!orderName || !deploymentKey) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}

	$: if ($registry && orderName) {
		const allDetails = $registry.getAllOrderDetails();
		if (!allDetails.error && allDetails.value) {
			const detail = allDetails.value.get(orderName);
			if (detail) {
				orderDetail = detail;
			}
		}
		const depDetails = $registry.getDeploymentDetails(orderName);
		if (!depDetails.error && depDetails.value) {
			const dep = depDetails.value.get(deploymentKey);
			if (dep) {
				deployment = { key: deploymentKey, name: dep.name, description: dep.description };
			}
		}
	}

	$: if ($registry && orderName && deploymentKey) {
		(async () => {
			const result = await $registry.getGui(
				orderName,
				deploymentKey,
				stateFromUrl,
				pushGuiStateToUrlHistory
			);
			if (result.error) {
				getGuiError = result.error.readableMsg ?? result.error.msg ?? 'Failed to build GUI';
				gui = null;
			} else {
				gui = result.value;
				getGuiError = null;
			}
		})();
	}

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

{#if $loading}
	<div>Loading deployment…</div>
{:else if $error}
	<div>Failed to initialize registry: {$error}</div>
{:else if !deployment}
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
