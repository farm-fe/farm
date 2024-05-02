/**
 * Returns the root directory name from the file path.
 *
 * @param file The file path string.
 * @param sep The separator, default is '/'
 * @returns The root directory name or null if not found.
 *
 * @example
 * ```ts
 * let rootDir = getRootDir('/src/utils/file.ts');
 * console.log(rootDir); // Output: 'src'
 */
export const getRootDir = (filePath: string, sep = "/"): string | null => {
  // Find the position of the first separator in the file path
  const idx = filePath?.indexOf(sep);
  if (idx === -1) {
    // If no separator is found, it means there is no directory in the path, return null
    return null;
  }
  if (idx === 0) {
    // If the separator is at the beginning of the path, it means the root directory is an empty string.
    // Then recursively call the getRootDir function to find the next separator.
    return sep + (getRootDir(filePath?.slice(1)) || "");
  }
  // If the separator is in the middle of the path, return the part from the beginning to the first separator, which is the root directory name.
  return filePath?.slice(0, idx);
};

export interface FileNode {
  title: string;
  key: string | number;
  children?: FileNode[];
  isLeaf?: boolean;
}

export function genFileTree(files: string[], sep = "/"): FileNode[] {
  const sepRegexp = new RegExp(sep);

  const res =
    files.reduce<FileNode>(
      (t, file) => {
        let dir = getRootDir(file, sep);
        let basename = dir ? file?.slice(dir.length + 1) : file;
        let parent: FileNode = t;

        while (dir) {
          let exist = parent.children?.find((e) => e.title === dir) as FileNode;
          if (!exist) {
            const p = [parent.key, dir].filter(Boolean).join(sep);
            exist = {
              title: dir,
              key: p,
              children: []
            };
            parent.children?.push(exist);
          }

          parent = exist;
          dir = getRootDir(basename);
          basename = dir
            ? basename.slice(dir.length).replace(sepRegexp, "")
            : basename;
        }

        if (parent.children?.some((e) => e.key === file)) return t;

        parent.children?.push({
          title: basename,
          key: file,
          isLeaf: true
        });

        return t;
      },
      { title: "", key: "", children: [] }
    ).children || [];

  res.forEach((fileNode) => {
    fileNode.children &&
      fileNode.children.forEach((child) =>
        flattenDirectory(child, fileNode, sep)
      );
  });

  return res;
}

/**
 * Flatten directory nodes for a more concise hierarchy in the file structure display.
 *
 * @param {FileNode} n - Current directory node.
 * @param {FileNode} parent - Parent directory node.
 * @param {string} [sep='/'] - File path separator, default is '/'.
 */
export function flattenDirectory(
  n: FileNode,
  parent: FileNode,
  sep = "/"
): void {
  // Skip flattening if the current node is a leaf node
  if (n.isLeaf) return;

  // Merge nodes if the parent has only one child
  if (parent.children && parent.children.length === 1) {
    // Generate a default title
    const defaultTitle = [parent.title, n.title].join(sep);

    // Update parent node's properties and children
    parent.children = n.children;
    parent.title = defaultTitle;

    // Recursively flatten child nodes
    n.children &&
      n.children.forEach((c) => {
        flattenDirectory(c, parent, sep);
      });
  } else {
    // Generate a default title
    const defaultTitle = [parent.title, n.title].join(sep);

    // If the parent has multiple children, only update the title of the current node
    n.title = defaultTitle;

    // Recursively flatten child nodes
    n.children &&
      n.children.forEach((c) => {
        flattenDirectory(c, n, sep);
      });
  }
}
