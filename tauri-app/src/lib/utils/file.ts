import { open, save } from '@tauri-apps/api/dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';

export async function loadFile(name: string, extension: string) {
  const path = await open({
    multiple: false,
    filters: [{
      name,
      extensions: [extension]
    }]
  });

  if(!path) {
    throw Error("No file selected");
  }

  const contents = await readTextFile(path as string);

  return [contents, path as string];
}

export async function saveFileAs(contents: string, name: string, extension: string) {
  const path = await save({
    filters: [{
      name,
      extensions: [extension]
    }]
  });

  if(!path) {
    throw Error("No file selected");
  }

  await writeTextFile(path as string, contents);

  return path as string;
}

export async function saveFile(contents: string, path: string) {
  await writeTextFile(path, contents);
}

export const loadDotrainFile = () => loadFile('Dotrain Orderbook Order', 'rain');

export const saveDotrainFileAs = (contents: string) => saveFileAs(contents, 'Dotrain Orderbook Order', 'rain');
