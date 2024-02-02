
export async function forkParseCallback(dotrain: RainDocument): Promise<any> {
  const composed = dotrain.compose(["calculate-order", "handle-io"]);
  const frontmatter = dotrain.frontMatter();
  // invoke tauri fork parse command
}