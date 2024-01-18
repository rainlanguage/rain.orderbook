export function isUrlValid(url: string) {
  try {
    console.log('url is ', url);
    new URL(url);
    return true;
  } catch (e) {
    return false;
  }
}