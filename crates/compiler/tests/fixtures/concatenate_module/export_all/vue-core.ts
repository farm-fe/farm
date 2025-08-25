function h(tag: string, props: Record<string, string>, children: string) {
  return {
    tag,
    props,
    children,
  }
}

function createBaseVNode() {
  return 'base vnode';
}

export { createBaseVNode as createElementVNode, h }