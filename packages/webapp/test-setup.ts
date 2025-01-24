import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock for codemirror-rainlang
vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));