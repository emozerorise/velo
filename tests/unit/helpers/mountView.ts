import { createRenderer, h, reactive, type Component } from "vue";
import type { Pinia } from "pinia";

export class TestNode extends EventTarget {
  children: TestNode[] = [];
  parent: TestNode | null = null;
  props: Record<string, unknown> = {};
  style: Record<string, unknown> = {};
  text = "";
  value = "";
  scrollTop = 0;
  scrollHeight = 0;
  clientHeight = 0;
  focusCount = 0;
  focus() {
    this.focusCount++;
    if (
      typeof document !== "undefined" &&
      Object.getOwnPropertyDescriptor(document, "activeElement")?.writable
    )
      Reflect.set(document, "activeElement", this);
  }
  get tabIndex() {
    return Number(
      this.props.tabindex ??
        (["BUTTON", "INPUT", "SELECT", "TEXTAREA", "A"].includes(this.tagName)
          ? 0
          : -1),
    );
  }
  get isConnected(): boolean {
    if (!this.parent) return this.tagName === "ROOT" || this.tagName === "BODY";
    return this.parent.children.includes(this) && this.parent.isConnected;
  }
  hasAttribute(key: string) {
    return (
      key in this.props &&
      this.props[key] !== false &&
      this.props[key] !== undefined
    );
  }
  getClientRects() {
    const visible = (node: TestNode): boolean =>
      node.style.display !== "none" && (!node.parent || visible(node.parent));
    return this.isConnected && visible(this)
      ? [this.getBoundingClientRect()]
      : [];
  }
  querySelectorAll<T = TestNode>(selector: string): T[] {
    const all = (node: TestNode): TestNode[] =>
      node.children.flatMap((child) => [child, ...all(child)]);
    const nodes = all(this).filter((node) =>
      selector === "[autofocus]"
        ? node.hasAttribute("autofocus")
        : ["BUTTON", "INPUT", "SELECT", "TEXTAREA"].includes(node.tagName) ||
          node.hasAttribute("href") ||
          node.hasAttribute("tabindex"),
    );
    return nodes as unknown as T[];
  }
  querySelector<T = TestNode>(selector: string): T | null {
    return this.querySelectorAll<T>(selector)[0] ?? null;
  }
  constructor(public tagName = "") {
    super();
  }
  scrollIntoView() {}
  getRootNode(): TestNode {
    return this.parent ? this.parent.getRootNode() : this;
  }
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
  for (const fn of handlers) if (typeof fn === "function") fn(input);
  if (node.parent && !(input instanceof Event && input.cancelBubble))
    bubble(node.parent, event, input);
}

const teleportRoot = new TestNode("BODY");
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
  const root = new TestNode("ROOT");
  const inputProps = reactive({ ...props });
  const app = renderer
    .createApp({ render: () => h(component, { ...inputProps }) })
    .use(pinia);
  app.mount(root);
  function all(node: TestNode): TestNode[] {
    return [node, ...node.children.flatMap(all)];
  }
  return {
    updateProps: (next: Record<string, unknown>) =>
      Object.assign(inputProps, next),
    nodes: () => [...all(root), ...all(teleportRoot)],
    find(testId: string): TestNode {
      const found = [...all(root), ...all(teleportRoot)].find(
        (node) => node.props["data-testid"] === testId,
      );
      if (!found) throw new Error(`Missing test node: ${testId}`);
      return found;
    },
    unmount: () => app.unmount(),
  };
}
