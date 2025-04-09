import { describe, expect, it, beforeEach, vi } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import Layout from './+layout.svelte';
import RegistryManager from '$lib/services/RegistryManager';
import type { Mock } from 'vitest';
import {
	type RegistryDotrain
} from '@rainlanguage/ui-components/services';
import type { ValidStrategyDetail } from '@rainlanguage/ui-components';
import type { InvalidStrategyDetail } from '@rainlanguage/ui-components';

const { mockPageStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));

vi.mock('$app/stores', () => {
	return { page: mockPageStore };
});

vi.mock('$lib/services/RegistryManager', () => ({
	default: {
		setToStorage: vi.fn(),
		clearFromStorage: vi.fn(),
		isCustomRegistry: vi.fn(),
		getFromStorage: vi.fn(),
		updateUrlParam: vi.fn()
	}
}));

vi.mock('@rainlanguage/ui-components/services', () => ({
	validateStrategies: vi.fn(),
	fetchRegistryDotrains: vi.fn()
}));

type LoadResult = {
	registry: string;
	registryDotrains: RegistryDotrain[];
	validStrategies: ValidStrategyDetail[];
	invalidStrategies: InvalidStrategyDetail[];
	error: string | null;
};

const mockDotrains = ['dotrain1', 'dotrain2'] as unknown as RegistryDotrain[];
const mockValidated = {
	validStrategies: ['strategy1', 'strategy2'] as unknown as ValidStrategyDetail[],
	invalidStrategies: ['invalidStrategy'] as unknown as InvalidStrategyDetail[]
};

describe('Layout Component', () => {
	beforeEach((): void => {
		vi.clearAllMocks();
		mockPageStore.reset();
		vi.unstubAllGlobals();
		vi.stubGlobal('localStorage', {
			getItem: vi.fn().mockReturnValue(null)
		});
	});

	it('should show advanced mode toggle on deploy page', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.getByText('Advanced mode')).toBeInTheDocument();
	});

	it('should not show advanced mode toggle on non-deploy pages', () => {
		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/other-page'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.queryByText('Advanced mode')).toBeNull();
	});

	it('should show custom registry warning when using non-default registry', () => {
		(RegistryManager.getFromStorage as Mock).mockReturnValue('https://custom-registry.com');
		(RegistryManager.isCustomRegistry as Mock).mockReturnValue(true);

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(RegistryManager.getFromStorage).toHaveBeenCalled();
		expect(screen.getByTestId('custom-registry-warning')).toBeTruthy();
	});

	it('should display InputRegistryUrl when advanced mode is on', () => {
		vi.stubGlobal('localStorage', {
			getItem: vi.fn().mockReturnValue('https://custom-registry.com')
		});

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.getByTestId('registry-input')).toBeTruthy();
	});

	it('should not display InputRegistryUrl when advanced mode is off', () => {
		vi.stubGlobal('localStorage', {
			getItem: vi.fn().mockReturnValue(null)
		});

		mockPageStore.mockSetSubscribeValue({
			url: {
				pathname: '/deploy'
			} as unknown as URL
		});

		render(Layout);

		expect(screen.queryByTestId('registry-input')).toBeNull();
	});
});
