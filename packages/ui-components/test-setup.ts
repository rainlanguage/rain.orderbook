import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('$app/stores', async () => {
	const { readable, writable } = await import('svelte/store');
	/**
	 * @type {import('$app/stores').getStores}
	 */
	const getStores = () => ({
		navigating: readable(null),
		page: readable({
			url: new URL('http://localhost'),
			params: {},
			searchParams: new URLSearchParams()
		}),
		session: writable(null),
		updated: readable(false)
	});
	/** @type {typeof import('$app/stores').page} */
	const page = {
		subscribe(fn) {
			return getStores().page.subscribe(fn);
		}
	};
	/** @type {typeof import('$app/stores').navigating} */
	const navigating = {
		subscribe(fn) {
			return getStores().navigating.subscribe(fn);
		}
	};
	/** @type {typeof import('$app/stores').session} */
	const session = {
		subscribe(fn) {
			return getStores().session.subscribe(fn);
		}
	};
	/** @type {typeof import('$app/stores').updated} */
	const updated = {
		subscribe(fn) {
			return getStores().updated.subscribe(fn);
		}
	};
	return {
		getStores,
		navigating,
		page,
		session,
		updated
	};
});

vi.mock('@rainlanguage/orderbook/js_api', () => {
	const DotrainOrderGui = vi.fn();
	// @ts-expect-error - static method
	DotrainOrderGui.getStrategyDetails = vi.fn();
	// @ts-expect-error - static method
	DotrainOrderGui.getDeploymentDetails = vi.fn();
	DotrainOrderGui.prototype.setVaultId = vi.fn();
	DotrainOrderGui.prototype.getTokenInfo = vi.fn();
	DotrainOrderGui.prototype.getCurrentDeployment = vi.fn();
	DotrainOrderGui.prototype.getVaultIds = vi.fn();
	DotrainOrderGui.prototype.saveDeposit = vi.fn();
	DotrainOrderGui.prototype.isDepositPreset = vi.fn();
	DotrainOrderGui.prototype.getDeposits = vi.fn();
	return {
		DotrainOrderGui
	};
});
