const REMOTE_SETTINGS_URL =
  'https://raw.githubusercontent.com/rainlanguage/rain.strategies/1321342555262ff6867f888957b3f03421df334f/settings.yaml';

export async function loadRemoteSettings() {
  const response = await fetch(REMOTE_SETTINGS_URL);
  if (!response.ok) {
    throw new Error('Error status: ' + response.status.toString());
  }
  return response.text();
}

if (import.meta.vitest) {
  const { describe, it, expect, vi, beforeEach, afterEach } = import.meta.vitest;

  // Mock fetch globally
  const mockFetch = vi.fn();
  global.fetch = mockFetch;

  describe('loadRemoteSettings', () => {
    beforeEach(() => {
      vi.clearAllMocks();
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it('should load the remote settings successfully', async () => {
      const mockText = vi.fn().mockResolvedValue('mock-settings-yaml');
      mockFetch.mockResolvedValue({
        ok: true,
        text: mockText,
      });

      const result = await loadRemoteSettings();

      expect(mockFetch).toHaveBeenCalledWith(REMOTE_SETTINGS_URL);
      expect(mockText).toHaveBeenCalled();
      expect(result).toBe('mock-settings-yaml');
    });

    it('should throw an error when fetch fails with non-ok status', async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 404,
      });

      await expect(loadRemoteSettings()).rejects.toThrow('Error status: 404');

      expect(mockFetch).toHaveBeenCalledWith(REMOTE_SETTINGS_URL);
    });

    it('should throw an error when fetch rejects', async () => {
      const networkError = new Error('Network error');
      mockFetch.mockRejectedValue(networkError);

      await expect(loadRemoteSettings()).rejects.toThrow('Network error');

      expect(mockFetch).toHaveBeenCalledWith(REMOTE_SETTINGS_URL);
    });

    it('should handle response.text() throwing an error', async () => {
      const textError = new Error('Text parsing error');
      const mockText = vi.fn().mockRejectedValue(textError);

      mockFetch.mockResolvedValue({
        ok: true,
        text: mockText,
      });

      await expect(loadRemoteSettings()).rejects.toThrow('Text parsing error');

      expect(mockFetch).toHaveBeenCalled();
      expect(mockText).toHaveBeenCalled();
    });
  });
}
