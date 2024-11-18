export function isUrlValid(url: string) {
  try {
    new URL(url);
    return true;
  } catch (_error) {
    return false;
  }
}
