import { createRenderer, type Component } from 'vue';
import type { Pinia } from 'pinia';

export class TestNode extends EventTarget {
  children: TestNode[] = [];
  parent: TestNode | null = null;
  props: Record<string, unknown> = {};
  style: Record<string, unknown> = {};
  text = '';
  value = '';
  scrollTop = 0;
  focusCount = 0;
  focus() {
    this.focusCount++;
  }
  constructor(public tagName = '') {
    super();
  }
  scrollIntoView() {}
  getBoundingClientRect() {
    return { x: 0, y: 0, width: 320, height: 180 };
  }
  setAttribute(key: string, value: unknown) {
    this.props[key] = value;
  }
  removeAttribute(key: string) {
    delete this.props[key];
  }
  trigger(event: string, payload?: unknown) {
    const input = payload ?? new Event(event, { cancelable: true });
    if (input instanceof Event) this.dispatchEvent(input);
    bubble(this, event, input);
    return input;
  }
}

function bubble(node: TestNode, event: string, input: unknown) {
  const handler = node.props[`on${event[0].toUpperCase()}${event.slice(1)}`];
  const handlers = Array.isArray(handler) ? handler : [handler];
  for (const fn of handlers) if (typeof fn === 'function') fn(input);
  if (node.parent && !(input instanceof Event && input.cancelBubble))
    bubble(node.parent, event, input);
}

const teleportRoot = new TestNode('BODY');
const renderer = createRenderer<TestNode, TestNode>({
  querySelector: () => teleportRoot,
  createElement: (tag) => new TestNode(tag.toUpperCase()),
  createText: (text) => Object.assign(new TestNode(), { text }),
  createComment: () => new TestNode(),
  setText(node, text) {
    node.text = text;
  },
  setElementText(node, text) {
    node.text = text;
    node.children = [];
  },
  patchProp(node, key, _old, value) {
    node.props[key] = value;
  },
  insert(child, parent, anchor) {
    if (child.parent)
      child.parent.children.splice(child.parent.children.indexOf(child), 1);
    const index = anchor ? parent.children.indexOf(anchor) : -1;
    parent.children.splice(
      index < 0 ? parent.children.length : index,
      0,
      child,
    );
    child.parent = parent;
  },
  remove(child) {
    if (child.parent)
      child.parent.children.splice(child.parent.children.indexOf(child), 1);
    child.parent = null;
  },
  parentNode: (node) => node.parent,
  nextSibling: (node) =>
    node.parent?.children[node.parent.children.indexOf(node) + 1] ?? null,
});

export function mountView(
  component: Component,
  props: Record<string, unknown>,
  pinia: Pinia,
) {
  teleportRoot.children = [];
  const root = new TestNode();
  const app = renderer.createApp(component, props).use(pinia);
  app.mount(root);
  function all(node: TestNode): TestNode[] {
    return [node, ...node.children.flatMap(all)];
  }
  return {
    nodes: () => all(root),
    find(testId: string): TestNode {
      const found = all(root).find(
        (node) => node.props['data-testid'] === testId,
      );
      if (!found) throw new Error(`Missing test node: ${testId}`);
      return found;
    },
    unmount: () => app.unmount(),
  };
}
