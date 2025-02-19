export interface NodeOptions {
  filename: string;
  size: number;
}

export type GroupNode<T = {}> = {
  filename: string;
  label: string;
  size: number;
  groups: GroupNode<T>[];
} & T;

export class Node<T = {}> {
  filenmae: string;
  size: number;
  children: Map<string, Node<T>>;
  isEndOfPath: boolean;
  groups: Array<GroupNode<T>>;
  constructor(options: Partial<NodeOptions>) {
    this.filenmae = options.filename || '';
    this.size = options.size || 0;
    this.children = new Map();
    this.groups = [];
    this.isEndOfPath = false;
  }
}

export type TrieWalkHandler<T = {}> = {
  enter: (child: GroupNode<T>, parent: Node<T>) => void;
  leave: (child: GroupNode<T>, parent: Node<T>) => void;
};

export class Trie<T = {}> {
  root: Node<T>;
  constructor(options: Partial<NodeOptions>) {
    this.root = new Node(options);
  }
  insert(filePath: string, opt: Partial<NodeOptions>) {
    let current = this.root;
    const dirs = filePath.split('/').filter(Boolean);
    let p = '';
    for (const dir of dirs) {
      p = p ? `${p}/${dir}` : dir;
      if (!current.children.has(dir)) {
        current.children.set(dir, new Node({ ...opt }));
      }
      current = current.children.get(dir);
      current.filenmae = p;
    }
    current.isEndOfPath = true;
  }
  mergeUniqueNode(node = this.root) {
    for (const [key, childNode] of node.children.entries()) {
      if (childNode.isEndOfPath) {
        break;
      }
      if (childNode.children.size > 1) {
        this.mergeUniqueNode(childNode);
        continue;
      }
      node.children.delete(key);
      for (const [subKey, subNode] of childNode.children.entries()) {
        node.children.set(`${key}/${subKey}`, subNode);
        if (!subNode.isEndOfPath) {
          this.mergeUniqueNode(subNode);
        }
      }
    }
  }
  walk(node: Node<T>, handler: TrieWalkHandler<T>) {
    if (!node.children.size) return;
    for (const [id, cn] of node.children.entries()) {
      const c = {
        size: cn.size,
        label: id,
        filename: cn.filenmae,
        groups: cn.groups
      } as GroupNode<T>;
      if (cn.isEndOfPath) {
        delete c.groups;
      }
      handler.enter(c, node);
      this.walk(cn, handler);
      if (cn.groups && cn.groups.length) {
        handler.leave(c, node);
      }
    }
    node.children.clear();
  }
}
