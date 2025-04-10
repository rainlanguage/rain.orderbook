import { DotrainOrderGui, type WasmEncodedResult } from '@rainlanguage/orderbook';
import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

export async function handleGuiInitialization(
	dotrain: string,
	deploymentKey: string,
	stateFromUrl: string | null
): Promise<{ gui: DotrainOrderGui | null; error: string | null }> {
	try {
		const gui = new DotrainOrderGui();
		let result: WasmEncodedResult<void>;

		if (stateFromUrl) {
			try {
				result = await gui.deserializeState(dotrain, stateFromUrl, pushGuiStateToUrlHistory);
				if (result.error) {
					throw new Error(result.error.msg);
				}
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
