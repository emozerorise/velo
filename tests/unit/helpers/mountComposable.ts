import { createRenderer, defineComponent } from 'vue';

type HostNode = { children: HostNode[]; parent: HostNode | null };
const node = (): HostNode => ({ children: [], parent: null });

// Composables need real mount/unmount hooks, not a browser layout engine.
const renderer = createRenderer<HostNode, HostNode>({
  createElement: node,
  createText: node,
  createComment: node,
  setText() {},
  setElementText() {},
  patchProp() {},
  insert(child, parent) {
    parent.children.push(child);
    child.parent = parent;
  },
  remove(child) {
    if (child.parent) child.parent.children.splice(child.parent.children.indexOf(child), 1);
  },
  parentNode: (child) => child.parent,
  nextSibling: () => null,
});

export function mountComposable(setup: () => void): () => void {
  const app = renderer.createApp(
    defineComponent({
      setup() {
        setup();
        return () => null;
      },
    }),
  );
  app.mount(node());
  return () => app.unmount();
}
