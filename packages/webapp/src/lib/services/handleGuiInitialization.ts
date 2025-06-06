import { DotrainOrderGui, type WasmEncodedResult } from '@rainlanguage/orderbook';
import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

export async function handleGuiInitialization(
	dotrain: string,
	deploymentKey: string,
	stateFromUrl: string | null
): Promise<{ gui: DotrainOrderGui | null; error: string | null }> {
	try {
		let gui: DotrainOrderGui;
		let result: WasmEncodedResult<DotrainOrderGui>;

		if (stateFromUrl) {
			try {
				result = await DotrainOrderGui.deserializeState(
					dotrain,
					stateFromUrl,
					pushGuiStateToUrlHistory
				);
				if (result.error) {
					throw new Error(result.error.msg);
				}
				gui = result.value;
			} catch {
				result = await DotrainOrderGui.chooseDeployment(
					dotrain,
					deploymentKey,
					pushGuiStateToUrlHistory
				);
				if (result.error) {
					throw new Error(result.error.msg);
				}
				gui = result.value;
			}
		} else {
			result = await DotrainOrderGui.chooseDeployment(
				dotrain,
				deploymentKey,
				pushGuiStateToUrlHistory
			);
			if (result.error) {
				throw new Error(result.error.msg);
			}
			gui = result.value;
		}

		return { gui, error: null };
	} catch {
		return { gui: null, error: 'Could not get deployment form.' };
	}
}
