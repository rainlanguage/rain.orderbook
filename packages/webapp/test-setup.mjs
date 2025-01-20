import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

vi.mock('codemirror-rainlang', () => ({
	RainlangLR: vi.fn()
}));

vi.mock('@walletconnect/relay-auth', () => ({
	utils: vi.fn()
}));

vi.mock('@walletconnect', () => ({
	default: vi.fn()
}));

vi.mock('uint8arrays/concat', () => ({
	concat: vi.fn()
}));
