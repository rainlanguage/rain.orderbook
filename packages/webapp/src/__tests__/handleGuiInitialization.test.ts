import { describe, it, expect, beforeEach, vi } from 'vitest';
import { handleGuiInitialization } from '../lib/services/handleGuiInitialization';
import { pushGuiStateToUrlHistory } from '../lib/services/handleUpdateGuiState';

const mockDotrainOrderGui = await vi.hoisted(() => ({
	deserializeState: vi.fn(),
	chooseDeployment: vi.fn()
}));

vi.mock('@rainlanguage/orderbook/js_api', () => ({
	DotrainOrderGui: mockDotrainOrderGui
}));

describe('handleGuiInitialization', () => {
	const mockDotrain = 'mockDotrain';
	const mockDeploymentKey = 'mockDeploymentKey';
	const mockGui = { id: 'mockGui' };

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should initialize GUI with state from URL when valid', async () => {
		mockDotrainOrderGui.deserializeState.mockResolvedValue(mockGui);

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'validStateUrl');

		expect(result).toEqual({ gui: mockGui, error: null });
		expect(mockDotrainOrderGui.deserializeState).toHaveBeenCalledWith(
			mockDotrain,
			'validStateUrl',
			pushGuiStateToUrlHistory
		);
		expect(mockDotrainOrderGui.chooseDeployment).not.toHaveBeenCalled();
	});

	it('should fall back to chooseDeployment when deserializeState fails', async () => {
		mockDotrainOrderGui.deserializeState.mockRejectedValue(new Error('deserialize failed'));
		mockDotrainOrderGui.chooseDeployment.mockResolvedValue(mockGui);

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'invalidStateUrl');

		expect(result).toEqual({ gui: mockGui, error: null });
		expect(mockDotrainOrderGui.deserializeState).toHaveBeenCalled();
		expect(mockDotrainOrderGui.chooseDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should use chooseDeployment when no state URL is provided', async () => {
		mockDotrainOrderGui.chooseDeployment.mockResolvedValue(mockGui);

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({ gui: mockGui, error: null });
		expect(mockDotrainOrderGui.deserializeState).not.toHaveBeenCalled();
		expect(mockDotrainOrderGui.chooseDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should handle errors and return error message', async () => {
		mockDotrainOrderGui.chooseDeployment.mockRejectedValue(new Error('deployment failed'));

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({
			gui: null,
			error: 'Could not get deployment form.'
		});
	});
});
