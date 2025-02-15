import { pushState } from '$app/navigation';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { debounce } from 'lodash';

export function handleUpdateGuiState(gui: DotrainOrderGui) {
	pushGuiStateToUrlHistory(gui);
}

const pushGuiStateToUrlHistory = debounce((gui: DotrainOrderGui) => {
	const serializedState = gui.serializeState();
	if (serializedState) {
		pushState(`?state=${serializedState}`, { serializedState });
	}
}, 1000);

if (import.meta.vitest) {
	const { describe, it, expect, vi } = import.meta.vitest;

	// Mock pushState
	vi.mock('$app/navigation', () => ({
		pushState: vi.fn()
	}));

	describe('handleUpdateGuiState', () => {
		beforeEach(() => {
			vi.clearAllMocks();
			vi.useFakeTimers();
		});

		afterEach(() => {
			vi.useRealTimers();
		});

		it('should push state to URL history when serializedState exists', async () => {
			const mockSerializedState = 'mockSerializedState123';
			const mockGui = {
				serializeState: vi.fn().mockReturnValue(mockSerializedState)
			} as unknown as DotrainOrderGui;

			handleUpdateGuiState(mockGui);

			// Fast-forward timers to trigger debounced function
			await vi.advanceTimersByTimeAsync(1000);

			expect(pushState).toHaveBeenCalledWith(`?state=${mockSerializedState}`, {
				serializedState: mockSerializedState
			});
		});

		it('should not push state when serializedState is falsy', async () => {
			const mockGui = {
				serializeState: vi.fn().mockReturnValue(null)
			} as unknown as DotrainOrderGui;

			handleUpdateGuiState(mockGui);

			await vi.advanceTimersByTimeAsync(1000);

			expect(pushState).not.toHaveBeenCalled();
		});

		it('should debounce multiple calls', async () => {
			const mockSerializedState = 'mockSerializedState123';
			const mockGui = {
				serializeState: vi.fn().mockReturnValue(mockSerializedState)
			} as unknown as DotrainOrderGui;

			// Call multiple times in quick succession
			handleUpdateGuiState(mockGui);
			handleUpdateGuiState(mockGui);
			handleUpdateGuiState(mockGui);

			await vi.advanceTimersByTimeAsync(1000);

			// Should only be called once due to debouncing
			expect(pushState).toHaveBeenCalledTimes(1);
			expect(pushState).toHaveBeenCalledWith(`?state=${mockSerializedState}`, {
				serializedState: mockSerializedState
			});
		});
	});
}
