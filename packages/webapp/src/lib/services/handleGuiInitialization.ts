import { DotrainOrderGui } from '@rainlanguage/orderbook';
import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

export async function handleGuiInitialization(
	dotrain: string,
	deploymentKey: string,
	stateFromUrl: string | null
): Promise<{ gui: DotrainOrderGui | null; error: string | null }> {
	if (stateFromUrl) {
		const stateResult = await DotrainOrderGui.newFromState(
			dotrain,
			stateFromUrl,
			pushGuiStateToUrlHistory
		);

		if (!stateResult.error) {
			return { gui: stateResult.value, error: null };
		}

		// Fallback to newWithDeployment if newFromState fails
		const deploymentResult = await DotrainOrderGui.newWithDeployment(
			dotrain,
			deploymentKey,
			pushGuiStateToUrlHistory
		);

		if (deploymentResult.error)
			return {
				gui: null,
				error: `Failed to create deployment form: ${deploymentResult.error.readableMsg}`
			};

		return { gui: deploymentResult.value, error: null };
	} else {
		const result = await DotrainOrderGui.newWithDeployment(
			dotrain,
			deploymentKey,
			pushGuiStateToUrlHistory
		);

		if (result.error)
			return { gui: null, error: `Failed to create deployment form: ${result.error.readableMsg}` };

		return { gui: result.value, error: null };
	}
}
