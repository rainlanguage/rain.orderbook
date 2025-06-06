import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { handleGuiInitialization } from '../lib/services/handleGuiInitialization';
import { pushGuiStateToUrlHistory } from '../lib/services/handleUpdateGuiState';
import { DotrainOrderGui } from '@rainlanguage/orderbook';

vi.mock('@rainlanguage/orderbook', () => {
	const DotrainOrderGui = vi.fn();
	// @ts-expect-error static method
	DotrainOrderGui.deserializeState = vi.fn();
	// @ts-expect-error static method
	DotrainOrderGui.chooseDeployment = vi.fn();
	return {
		DotrainOrderGui
	};
});

describe('handleGuiInitialization', () => {
	const mockDotrain = 'mockDotrain';
	const mockDeploymentKey = 'mockDeploymentKey';

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should initialize GUI with state from URL when valid', async () => {
		(DotrainOrderGui.deserializeState as Mock).mockImplementation(() => ({ value: {} }));
		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'validStateUrl');

		expect(result).toEqual({ gui: {}, error: null });
		expect(DotrainOrderGui.deserializeState).toHaveBeenCalledWith(
			mockDotrain,
			'validStateUrl',
			pushGuiStateToUrlHistory
		);
		expect(DotrainOrderGui.chooseDeployment).not.toHaveBeenCalled();
	});

	it('should fall back to chooseDeployment when deserializeState fails', async () => {
		(DotrainOrderGui.deserializeState as Mock).mockReturnValue({
			error: { msg: 'deserialize failed' }
		});
		(DotrainOrderGui.chooseDeployment as Mock).mockResolvedValue({
			value: {}
		});

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'invalidStateUrl');

		expect(result).toEqual({ gui: {}, error: null });
		expect(DotrainOrderGui.deserializeState).toHaveBeenCalled();
		expect(DotrainOrderGui.chooseDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should use chooseDeployment when no state URL is provided', async () => {
		(DotrainOrderGui.chooseDeployment as Mock).mockResolvedValue({
			value: {}
		});

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({ gui: {}, error: null });
		expect(DotrainOrderGui.deserializeState).not.toHaveBeenCalled();
		expect(DotrainOrderGui.chooseDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should handle errors and return error message', async () => {
		(DotrainOrderGui.chooseDeployment as Mock).mockReturnValue({
			error: { msg: 'deployment failed' }
		});

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({
			gui: null,
			error: 'Could not get deployment form.'
		});
	});
});
