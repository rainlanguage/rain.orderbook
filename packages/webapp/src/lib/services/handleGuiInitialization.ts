import { DotrainOrderGui, type WasmEncodedResult } from '@rainlanguage/orderbook';
import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

export async function handleGuiInitialization(
	dotrain: string,
	deploymentKey: string,
	stateFromUrl: string | null
): Promise<{ gui: DotrainOrderGui | null; error: string | null }> {
	if (stateFromUrl) {
		const result = await DotrainOrderGui.newFromState(
			dotrain,
			stateFromUrl,
			pushGuiStateToUrlHistory
		);

		if (result.error)
			return { gui: null, error: `Failed to create deployment form: ${result.error.readableMsg}` };

		return { gui: result.value, error: null };
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
