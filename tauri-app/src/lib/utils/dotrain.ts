import { open } from '@tauri-apps/api/dialog';
import { readTextFile } from '@tauri-apps/api/fs';

export async function loadDotrainFile() {
  const path = await open({
    multiple: false,
    filters: [{
      name: 'Dotrain Orderbook Order',
      extensions: ['order.rain']
    }]
  });

  if(!path) {
    throw Error("No file selected");
  }

  const contents = await readTextFile(path as string);

  return contents;
}