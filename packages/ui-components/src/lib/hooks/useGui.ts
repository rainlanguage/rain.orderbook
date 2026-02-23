import { getContext } from 'svelte';
import { RaindexOrderBuilder } from '@rainlanguage/orderbook';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../errors/DeploymentStepsError';
export const GUI_CONTEXT_KEY = 'gui-context';

export function useGui(): RaindexOrderBuilder {
	const gui = getContext<RaindexOrderBuilder>(GUI_CONTEXT_KEY);
	if (!gui) {
		DeploymentStepsError.catch(null, DeploymentStepsErrorCode.NO_GUI_PROVIDER);
	}
	return gui;
}
