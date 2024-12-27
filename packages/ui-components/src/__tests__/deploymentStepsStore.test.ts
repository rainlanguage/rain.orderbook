import { get } from 'svelte/store';
import deploymentStepsStore from '../lib/components/deployment/wizard/deploymentStepsStore'
import type { WizardStep, TokenInputStep, FieldStep } from '../lib/types/wizardSteps';

describe('deploymentStepsStore', () => {
    beforeEach(() => {
        deploymentStepsStore.reset();
    });

    it('should initialize with empty array', () => {
        const steps = get(deploymentStepsStore);
        expect(steps).toEqual([]);
    });

    it('should populate steps correctly', () => {
        const mockSteps: WizardStep[] = [
            {
                type: 'tokenInput',
                input: {
                    token: '0x123...',
                    decimals: 18,
                    symbol: 'TEST'
                },
                gui: {
                    name: 'Test GUI',
                    description: 'Test Description',
                },
                tokenInfos: {
                },
                i: 0,
                inputVaultIds: ['vault1', 'vault2']
            } as unknown as TokenInputStep,
            {
                type: 'fields',
                fieldDefinition: {
                    name: 'Test Field',
                    type: 'uint256',
                },
                gui: {
                    name: 'Test GUI',
                    description: 'Test Description',
                }
            } as unknown as FieldStep
        ];

        deploymentStepsStore.populateDeploymentSteps(mockSteps);
        const steps = get(deploymentStepsStore);

        expect(steps).toEqual(mockSteps);
    });

    it('should update a specific step correctly', () => {
        const initialSteps: WizardStep[] = [
            {
                type: 'tokenInput',
                input: {
                    token: '0x123...',
                    decimals: 18,
                    symbol: 'TEST'
                },
                gui: {
                    name: 'Test GUI',
                    description: 'Test Description'
                },
                tokenInfos: {
                },
                i: 0,
                inputVaultIds: ['vault1']
            } as unknown as TokenInputStep
        ];

        deploymentStepsStore.populateDeploymentSteps(initialSteps);

        const updatedStep: TokenInputStep = {
            ...initialSteps[0],
            inputVaultIds: ['vault1', 'vault2']
        } as unknown as TokenInputStep;

        deploymentStepsStore.updateDeploymentStep(0, updatedStep);

        const steps = get(deploymentStepsStore);
        expect(steps[0]).toEqual(updatedStep);
    });

    it('should reset store to initial state', () => {
        const mockSteps: WizardStep[] = [
            {
                type: 'fields',
                fieldDefinition: {
                    name: 'Test Field',
                    type: 'uint256',
                },
                gui: {
                    name: 'Test GUI',
                    description: 'Test Description',
                }
            } as unknown as FieldStep
        ];

        deploymentStepsStore.populateDeploymentSteps(mockSteps);
        deploymentStepsStore.reset();

        const steps = get(deploymentStepsStore);
        expect(steps).toEqual([]);
    });
});