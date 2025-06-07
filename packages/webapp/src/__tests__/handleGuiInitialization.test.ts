import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { handleGuiInitialization } from '../lib/services/handleGuiInitialization';
import { pushGuiStateToUrlHistory } from '../lib/services/handleUpdateGuiState';
import { DotrainOrderGui } from '@rainlanguage/orderbook';

vi.mock('@rainlanguage/orderbook', () => {
	const DotrainOrderGui = vi.fn();
	// @ts-expect-error static method
	DotrainOrderGui.newFromState = vi.fn();
	// @ts-expect-error static method
	DotrainOrderGui.newWithDeployment = vi.fn();
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
		(DotrainOrderGui.newFromState as Mock).mockImplementation(() => ({ value: {} }));
		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'validStateUrl');

		expect(result).toEqual({ gui: {}, error: null });
		expect(DotrainOrderGui.newFromState).toHaveBeenCalledWith(
			mockDotrain,
			'validStateUrl',
			pushGuiStateToUrlHistory
		);
		expect(DotrainOrderGui.newWithDeployment).not.toHaveBeenCalled();
	});

	it('should fall back to newWithDeployment when newFromState fails', async () => {
		(DotrainOrderGui.newFromState as Mock).mockReturnValue({
			error: { msg: 'deserialize failed' }
		});
		(DotrainOrderGui.newWithDeployment as Mock).mockResolvedValue({
			value: {}
		});

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, 'invalidStateUrl');

		expect(result).toEqual({ gui: {}, error: null });
		expect(DotrainOrderGui.newFromState).toHaveBeenCalled();
		expect(DotrainOrderGui.newWithDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should use newWithDeployment when no state URL is provided', async () => {
		(DotrainOrderGui.newWithDeployment as Mock).mockResolvedValue({
			value: {}
		});

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({ gui: {}, error: null });
		expect(DotrainOrderGui.newFromState).not.toHaveBeenCalled();
		expect(DotrainOrderGui.newWithDeployment).toHaveBeenCalledWith(
			mockDotrain,
			mockDeploymentKey,
			pushGuiStateToUrlHistory
		);
	});

	it('should handle errors and return error message', async () => {
		(DotrainOrderGui.newWithDeployment as Mock).mockReturnValue({
			error: { msg: 'deployment failed' }
		});

		const result = await handleGuiInitialization(mockDotrain, mockDeploymentKey, null);

		expect(result).toEqual({
			gui: null,
			error: 'Could not get deployment form.'
		});
	});
});
