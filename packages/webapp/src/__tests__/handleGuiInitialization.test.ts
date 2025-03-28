import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { handleGuiInitialization } from '../lib/services/handleGuiInitialization';
import { pushGuiStateToUrlHistory } from '../lib/services/handleUpdateGuiState';
import { DotrainOrderGui, type WasmEncodedResult } from '@rainlanguage/orderbook/js_api';

describe('handleGuiInitialization', () => {
	let guiInstance: DotrainOrderGui;
	const mockDotrain = 'mockDotrain';
	const mockDeploymentKey = 'mockDeploymentKey';

	beforeEach(() => {
		vi.clearAllMocks();
		guiInstance = new DotrainOrderGui();
	});

	it('should initialize GUI with state from URL when valid', async () => {
		(DotrainOrderGui.prototype.deserializeState as Mock).mockImplementation(() => ({ value: {} }));
		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'validStateUrl');

		expect(result).toEqual({ gui: guiInstance, error: null });
		expect(DotrainOrderGui.prototype.deserializeState).toHaveBeenCalledWith(
			mockDotrain,
			'validStateUrl',
			pushGuiStateToUrlHistory
		);
		expect(guiInstance.chooseDeployment).not.toHaveBeenCalled();
	});

	it('should fall back to chooseDeployment when deserializeState fails', async () => {
		(DotrainOrderGui.prototype.deserializeState as Mock).mockReturnValue({
			error: { msg: 'deserialize failed' }
		});
		(DotrainOrderGui.prototype.chooseDeployment as Mock).mockResolvedValue(
			{} as WasmEncodedResult<void>
		);

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'invalidStateUrl');

		expect(result).toEqual({ gui: guiInstance, error: null });
		expect(DotrainOrderGui.prototype.deserializeState).toHaveBeenCalled();
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
		expect(DotrainOrderGui.prototype.deserializeState).not.toHaveBeenCalled();
		expect(guiInstance.chooseDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should handle errors and return error message', async () => {
		(guiInstance.chooseDeployment as Mock).mockReturnValue({
			error: { msg: 'deployment failed' }
		});

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({
			gui: null,
			error: 'Could not get deployment form.'
		});
	});
});
