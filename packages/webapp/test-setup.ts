import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

vi.mock('@reown/appkit', () => ({
	default: vi.fn()
}));
// Mock for codemirror-rainlang
vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('@rainlanguage/orderbook', async() => {
	const DotrainOrderGui = vi.fn();
	DotrainOrderGui.prototype.deserializeState = vi.fn();
	DotrainOrderGui.prototype.chooseDeployment = vi.fn();
	return {
		DotrainOrderGui,
		getRemoveOrderCalldata: vi.fn()
	};
});
