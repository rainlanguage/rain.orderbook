/* eslint-disable @typescript-eslint/no-explicit-any */
import { getDeploymentSteps } from '../lib/components/deployment/wizard/getDeploymentSteps';
import { describe, it, expect } from 'vitest';

describe('getDeploymentSteps', () => {
	const mockGui = {} as any;
	const mockTokenInfos = {} as any;

	it('should return empty array when no inputs provided', () => {
		const steps = getDeploymentSteps(
			new Map(),
			false,
			[],
			mockGui,
			[],
			[],
			[],
			[],
			[],
			mockTokenInfos
		);

		expect(steps).toEqual([]);
	});

	it('should include select token steps for limit strategy', () => {
		const selectTokens = new Map([['TOKEN1', 'address1']]);

		const steps = getDeploymentSteps(
			selectTokens,
			true,
			[],
			mockGui,
			[],
			[],
			[],
			[],
			[],
			mockTokenInfos
		);

		expect(steps[0]).toEqual({
			type: 'tokens',
			token: 'TOKEN1',
			gui: mockGui,
			selectTokens
		});
	});

	it('should not include select token steps when not limit strategy', () => {
		const selectTokens = new Map([['TOKEN1', 'address1']]);

		const steps = getDeploymentSteps(
			selectTokens,
			false,
			[],
			mockGui,
			[],
			[],
			[],
			[],
			[],
			mockTokenInfos
		);

		expect(steps).toEqual([]);
	});

	it('should include all step types in correct order', () => {
		const selectTokens = new Map([['TOKEN1', 'address1']]);
		const fieldDefinitions = [{ id: 'field1' }] as any;
		const deposits = [{ id: 'deposit1' }] as any;
		const inputs = [{ id: 'input1' }] as any;
		const outputs = [{ id: 'output1' }] as any;
		const inputVaultIds = ['vault1'];
		const outputVaultIds = ['vault2'];

		const steps = getDeploymentSteps(
			selectTokens,
			true,
			fieldDefinitions,
			mockGui,
			deposits,
			inputs,
			outputs,
			inputVaultIds,
			outputVaultIds,
			mockTokenInfos
		);

		expect(steps).toHaveLength(5);
		expect(steps[0].type).toBe('tokens');
		expect(steps[1].type).toBe('fields');
		expect(steps[2].type).toBe('deposits');
		expect(steps[3].type).toBe('tokenInput');
		expect(steps[4].type).toBe('tokenOutput');
	});
});
