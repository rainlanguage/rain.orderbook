import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';
import { init as initOrderbookPackage } from '@rainlanguage/orderbook/esm';

await initOrderbookPackage();

vi.mock('@reown/appkit', () => ({
	default: vi.fn()
}));
// Mock for codemirror-rainlang
vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));