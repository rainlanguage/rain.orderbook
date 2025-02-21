import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';

export async function handleGuiInitialization(
	dotrain: string,
	deploymentKey: string,
	stateFromUrl: string | null
): Promise<{ gui: DotrainOrderGui | null; error: string | null }> {
	try {
		let gui: DotrainOrderGui | null = null;

		if (stateFromUrl) {
			try {
				gui = await DotrainOrderGui.deserializeState(
					dotrain,
					stateFromUrl,
					pushGuiStateToUrlHistory
				);
			} catch (deserializeErr) {
				gui = await DotrainOrderGui.chooseDeployment(
					dotrain,
					deploymentKey,
					pushGuiStateToUrlHistory
				);
			}
		} else {
			gui = await DotrainOrderGui.chooseDeployment(
				dotrain,
				deploymentKey,
				pushGuiStateToUrlHistory
			);
		}
		return { gui, error: null };
	} catch (err) {
		return { gui: null, error: 'Could not get deployment form.' };
	}
}
