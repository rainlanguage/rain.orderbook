import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';
import { mockRegistryStore } from './src/lib/__mocks__/stores';
vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('$lib/providers/registry/useRegistry', () => ({
	useRegistry: vi.fn().mockReturnValue(mockRegistryStore)
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

vi.mock('@rainlanguage/orderbook', () => {
	const DotrainOrderGui = vi.fn();
	// @ts-expect-error - static method
	DotrainOrderGui.getStrategyDetails = vi.fn();
	// @ts-expect-error - static method
	DotrainOrderGui.getDeploymentDetails = vi.fn();
	DotrainOrderGui.prototype.chooseDeployment = vi.fn();
	DotrainOrderGui.prototype.getStrategyDetails = vi.fn();
	DotrainOrderGui.prototype.setVaultId = vi.fn();
	DotrainOrderGui.prototype.getTokenInfo = vi.fn();
	DotrainOrderGui.prototype.getCurrentDeployment = vi.fn();
	DotrainOrderGui.prototype.getVaultIds = vi.fn();
	DotrainOrderGui.prototype.saveDeposit = vi.fn();
	DotrainOrderGui.prototype.getDeposits = vi.fn();
	DotrainOrderGui.prototype.saveFieldValue = vi.fn();
	DotrainOrderGui.prototype.getFieldValue = vi.fn();
	DotrainOrderGui.prototype.getSelectTokens = vi.fn();
	DotrainOrderGui.prototype.getNetworkKey = vi.fn();
	DotrainOrderGui.prototype.getAllTokenInfos = vi.fn();
	DotrainOrderGui.prototype.getAllFieldDefinitions = vi.fn();
	DotrainOrderGui.prototype.isSelectTokenSet = vi.fn();
	DotrainOrderGui.prototype.saveSelectToken = vi.fn();
	DotrainOrderGui.prototype.hasAnyDeposit = vi.fn();
	DotrainOrderGui.prototype.hasAnyVaultId = vi.fn();
	DotrainOrderGui.prototype.areAllTokensSelected = vi.fn();
	DotrainOrderGui.prototype.getDeploymentTransactionArgs = vi.fn();
	DotrainOrderGui.prototype.generateApprovalCalldatas = vi.fn();
	DotrainOrderGui.prototype.serializeState = vi.fn();
	DotrainOrderGui.prototype.getAllGuiConfig = vi.fn();
	DotrainOrderGui.prototype.getCurrentDeploymentDetails = vi.fn();
	return {
		DotrainOrderGui
	};
});
