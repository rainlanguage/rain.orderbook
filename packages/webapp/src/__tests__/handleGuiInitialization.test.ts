import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { handleGuiInitialization } from '../lib/services/handleGuiInitialization';
import { pushGuiStateToUrlHistory } from '../lib/services/handleUpdateGuiState';
import { DotrainOrderGui, type WasmEncodedResult } from '@rainlanguage/orderbook/js_api';

describe('handleGuiInitialization', () => {
	let guiInstance: DotrainOrderGui;
	const mockDotrain = 'mockDotrain';
	const mockDeploymentKey = 'mockDeploymentKey';
	const mockGui = { id: 'mockGui' };

	beforeEach(() => {
		vi.clearAllMocks();
		guiInstance = new DotrainOrderGui();
	});

	it('should initialize GUI with state from URL when valid', async () => {
		(DotrainOrderGui.deserializeState as Mock).mockResolvedValue(mockGui);

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'validStateUrl');

		expect(result).toEqual({ gui: mockGui, error: null });
		expect(DotrainOrderGui.deserializeState).toHaveBeenCalledWith(
			mockDotrain,
			'validStateUrl',
			pushGuiStateToUrlHistory
		);
		expect(guiInstance.chooseDeployment).not.toHaveBeenCalled();
	});

	it('should fall back to chooseDeployment when deserializeState fails', async () => {
		(DotrainOrderGui.deserializeState as Mock).mockRejectedValue(new Error('deserialize failed'));
		(DotrainOrderGui.prototype.chooseDeployment as Mock).mockResolvedValue(
			{} as WasmEncodedResult<void>
		);

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'invalidStateUrl');

		expect(result).toEqual({ gui: guiInstance, error: null });
		expect(DotrainOrderGui.deserializeState).toHaveBeenCalled();
		expect(guiInstance.chooseDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should use chooseDeployment when no state URL is provided', async () => {
		(guiInstance.chooseDeployment as Mock).mockResolvedValue({} as WasmEncodedResult<void>);

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({ gui: guiInstance, error: null });
		expect(DotrainOrderGui.deserializeState).not.toHaveBeenCalled();
		expect(guiInstance.chooseDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should handle errors and return error message', async () => {
		(guiInstance.chooseDeployment as Mock).mockRejectedValue(new Error('deployment failed'));

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({
			gui: null,
			error: 'Could not get deployment form.'
		});
	});
});
