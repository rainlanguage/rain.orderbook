import { writable } from 'svelte/store';
import type { WizardStep } from '../../../types/wizardSteps';

const initialState: {
	deploymentSteps: WizardStep[];
} = {
	deploymentSteps: [],

};

const deploymentStore = () => {
	const { subscribe, set, update } = writable(initialState);
    const reset = () => set(initialState);
    // For getting an array of steps from the various input types (deposit, token, vault)
	const populateDeploymentSteps = (steps: WizardStep[]) => {
		set({ deploymentSteps: steps });
	};
    // For adding a property (binding) to the current step
	const updateDeploymentStep = (step: WizardStep) => {
		update((state) => ({ ...state, step }));
	};
	return {
		subscribe,
		reset,
		updateDeploymentStep,
		populateDeploymentSteps
	};
};

export default deploymentStore();
