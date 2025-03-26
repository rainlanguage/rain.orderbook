import { DotrainOrderGui, type WasmEncodedResult } from '@rainlanguage/orderbook/js_api';
import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

export async function handleGuiInitialization(
	dotrain: string,
	deploymentKey: string,
	stateFromUrl: string | null
): Promise<{ gui: DotrainOrderGui | null; error: string | null }> {
	try {
		let gui = new DotrainOrderGui();
		let result: WasmEncodedResult<void>;

		if (stateFromUrl) {
			try {
				gui = await DotrainOrderGui.deserializeState(
					dotrain,
					stateFromUrl,
					pushGuiStateToUrlHistory
				);
			} catch {
				result = await gui.chooseDeployment(dotrain, deploymentKey, pushGuiStateToUrlHistory);
				if (result.error) {
					throw new Error(result.error.msg);
				}
			}
		} else {
			result = await gui.chooseDeployment(dotrain, deploymentKey, pushGuiStateToUrlHistory);
			if (result.error) {
				throw new Error(result.error.msg);
			}
		}
		return { gui, error: null };
	} catch {
		return { gui: null, error: 'Could not get deployment form.' };
	}
}
