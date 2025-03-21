import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { getContext } from 'svelte';
import { useGui, GUI_CONTEXT_KEY } from './useGui';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../errors/DeploymentStepsError';

vi.mock('svelte', () => ({
	getContext: vi.fn()
}));

vi.spyOn(DeploymentStepsError, 'catch');

describe('useGui hook', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	afterEach(() => {
		vi.clearAllMocks();
	});

	it('should return GUI context when available', () => {
		const mockGui = {
			someMethod: vi.fn(),
			someProperty: 'value'
		};

		vi.mocked(getContext).mockReturnValue(mockGui);

		const result = useGui();

		expect(getContext).toHaveBeenCalledWith(GUI_CONTEXT_KEY);
		expect(result).toBe(mockGui);
	});

	it('should call DeploymentStepsError.catch when GUI context is not available', () => {
		vi.mocked(getContext).mockReturnValue(null);

		useGui();

		expect(DeploymentStepsError.catch).toHaveBeenCalledWith(
			null,
			DeploymentStepsErrorCode.NO_GUI_PROVIDER
		);
	});
});
