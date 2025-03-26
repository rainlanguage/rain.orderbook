import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

vi.mock('@reown/appkit', () => ({
	default: vi.fn()
}));
// Mock for codemirror-rainlang
vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('@rainlanguage/orderbook/js_api', () => {
	const DotrainOrderGui = vi.fn();
	// @ts-expect-error - this is a mock
	DotrainOrderGui.deserializeState = vi.fn();
	DotrainOrderGui.prototype.chooseDeployment = vi.fn();
	return {
		DotrainOrderGui
	};
});
