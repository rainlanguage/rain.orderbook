import { pushState } from '$app/navigation';
import { debounce } from 'lodash';

function computeUrlWithGuiState(serializedState: string): string | null {
	if (typeof window === 'undefined') {
		return null;
	}

	const url = new URL(window.location.href);
	if (serializedState) {
		url.searchParams.set('state', serializedState);
	} else {
		url.searchParams.delete('state');
	}

	return url.toString();
}

export const pushGuiStateToUrlHistory = debounce((serializedState: string) => {
	const nextUrl = computeUrlWithGuiState(serializedState);
	if (!nextUrl) {
		return;
	}

	pushState(nextUrl, { serializedState });
}, 1000);

if (import.meta.vitest) {
	const { describe, it, expect, vi, beforeEach, afterEach } = import.meta.vitest;

	// Mock pushState
	vi.mock('$app/navigation', () => ({
		pushState: vi.fn()
	}));

	describe('handleUpdateGuiState', () => {
		const originalHref = window.location.href;
		const origin = new URL(originalHref).origin;

		beforeEach(() => {
			vi.clearAllMocks();
			vi.useFakeTimers();
			window.history.replaceState({}, '', originalHref);
		});

		afterEach(() => {
			window.history.replaceState({}, '', originalHref);
			vi.useRealTimers();
		});

		it('should push state to URL history when serializedState exists', async () => {
			const mockSerializedState = 'mockSerializedState123';
			const baseUrl = `${origin}/deploy/some-order`;
			window.history.replaceState({}, '', baseUrl);

			pushGuiStateToUrlHistory(mockSerializedState);

			// Fast-forward timers to trigger debounced function
			await vi.advanceTimersByTimeAsync(1000);

			const [[urlArg, stateArg]] = vi.mocked(pushState).mock.calls;
			expect(typeof urlArg).toBe('string');
			expect(urlArg).toBe(`${baseUrl}?state=${mockSerializedState}`);
			expect(stateArg).toEqual({ serializedState: mockSerializedState });
		});

		it('should debounce multiple calls', async () => {
			const mockSerializedState = 'mockSerializedState123';
			const baseUrl = `${origin}/deploy/another-order`;
			window.history.replaceState({}, '', baseUrl);

			// Call multiple times in quick succession
			pushGuiStateToUrlHistory(mockSerializedState);
			pushGuiStateToUrlHistory(mockSerializedState);
			pushGuiStateToUrlHistory(mockSerializedState);

			await vi.advanceTimersByTimeAsync(1000);

			// Should only be called once due to debouncing
			expect(pushState).toHaveBeenCalledTimes(1);
			const [[urlArg, stateArg]] = vi.mocked(pushState).mock.calls;
			expect(urlArg).toBe(`${baseUrl}?state=${mockSerializedState}`);
			expect(stateArg).toEqual({ serializedState: mockSerializedState });
		});

		it('should preserve existing query params when updating state', async () => {
			const mockSerializedState = 'newState';
			const currentUrl = `${origin}/deploy/order?registry=foo&state=oldState`;
			window.history.replaceState({}, '', currentUrl);

			pushGuiStateToUrlHistory(mockSerializedState);

			await vi.advanceTimersByTimeAsync(1000);

			const [[urlArg, stateArg]] = vi.mocked(pushState).mock.calls;
			expect(urlArg).toBe(`${origin}/deploy/order?registry=foo&state=newState`);
			expect(stateArg).toEqual({ serializedState: mockSerializedState });
			expect(pushState).toHaveBeenCalledTimes(1);
		});

		it('should remove state query param when serializedState is empty', async () => {
			const currentUrl = `${origin}/deploy/order?state=oldState`;
			window.history.replaceState({}, '', currentUrl);

			pushGuiStateToUrlHistory('');

			await vi.advanceTimersByTimeAsync(1000);

			const [[urlArg, stateArg]] = vi.mocked(pushState).mock.calls;
			expect(urlArg).toBe(`${origin}/deploy/order`);
			expect(stateArg).toEqual({ serializedState: '' });
			expect(pushState).toHaveBeenCalledTimes(1);
		});
	});

	describe('computeUrlWithGuiState', () => {
		const originalHref = window.location.href;
		const origin = new URL(originalHref).origin;

		beforeEach(() => {
			window.history.replaceState({}, '', originalHref);
		});

		afterEach(() => {
			window.history.replaceState({}, '', originalHref);
		});

		it('should return null when window is undefined', () => {
			const originalWindow = globalThis.window;
			// @ts-expect-error intentionally unset for test
			globalThis.window = undefined;
			expect(computeUrlWithGuiState('test')).toBeNull();
			globalThis.window = originalWindow;
		});

		it('should append state parameter', () => {
			const baseUrl = `${origin}/foo`;
			window.history.replaceState({}, '', baseUrl);
			expect(computeUrlWithGuiState('abc')).toBe(`${baseUrl}?state=abc`);
		});

		it('should remove state parameter when empty', () => {
			const currentUrl = `${origin}/foo?state=old`;
			window.history.replaceState({}, '', currentUrl);
			expect(computeUrlWithGuiState('')).toBe(`${origin}/foo`);
		});
	});
}
