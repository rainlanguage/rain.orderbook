import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock for codemirror-rainlang
vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

Object.defineProperty(window, 'matchMedia', {
	writable: true,
	value: vi.fn().mockImplementation(query => ({
		matches: query === '(min-width: 1024px)',
		media: query,
		onchange: null,
		addListener: vi.fn(),
		removeListener: vi.fn(),
		addEventListener: vi.fn(),
		removeEventListener: vi.fn(),
		dispatchEvent: vi.fn(),
	})),
});