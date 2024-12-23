import { writable } from 'svelte/store';
import type { WizardStep } from '../../../types/wizardSteps';

const initialState: WizardStep[] = []

const deploymentStepsStore = () => {
	const { subscribe, set, update } = writable(initialState);
    const reset = () => set(initialState);

    // For getting an array of steps from the various input types (deposit, token, vault)
	const populateDeploymentSteps = (steps: WizardStep[]) => {
		update(() => ( steps ));
	};

    // For adding a property (binding) to the current step
	const updateDeploymentStep = (index: number, updatedStep: WizardStep) => {
        update((state) => {
            const newSteps = [...state];
            newSteps[index] = updatedStep;
            return newSteps
        });
	};

	return {
		subscribe,
		reset,
		updateDeploymentStep,
		populateDeploymentSteps
	};
};

export default deploymentStepsStore();
