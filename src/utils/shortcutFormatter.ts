/**
 * 格式化快捷键字符串为可读的显示格式
 * 例如: "Cmd+Shift+S" -> { symbols: ["⌘", "⇧", "S"], readable: "按住 ⌘ Cmd + ⇧ Shift + S" }
 */

export interface ShortcutDisplay {
  symbols: string[];
  readable: string;
  keys: string[];
}

const keySymbolMap: Record<string, string> = {
  'Cmd': '⌘',
  'Command': '⌘',
  'Ctrl': '⌃',
  'Control': '⌃',
  'Alt': '⌥',
  'Opt': '⌥',
  'Option': '⌥',
  'Shift': '⇧',
  'CapsLock': '⇪',
  'Tab': '⇥',
  'Enter': '↩',
  'Return': '↩',
  'Delete': '⌫',
  'Backspace': '⌫',
  'Esc': '⎋',
  'Escape': '⎋',
  'Space': '␣',
  'Up': '↑',
  'Down': '↓',
  'Left': '←',
  'Right': '→',
};

const keyNameMap: Record<string, string> = {
  'Cmd': 'Cmd',
  'Command': 'Cmd',
  'Ctrl': 'Ctrl',
  'Control': 'Ctrl',
  'Alt': 'Opt',
  'Opt': 'Opt',
  'Option': 'Opt',
  'Shift': 'Shift',
};

export function formatShortcut(shortcut: string): ShortcutDisplay {
  if (!shortcut) {
    return {
      symbols: [],
      readable: '',
      keys: [],
    };
  }

  // 分割快捷键字符串
  const parts = shortcut.split('+').map(s => s.trim());

  const symbols: string[] = [];
  const readable: string[] = [];
  const keys: string[] = [];

  parts.forEach(part => {
    // 获取符号
    const symbol = keySymbolMap[part] || part.toUpperCase();
    symbols.push(symbol);

    // 获取可读名称
    const name = keyNameMap[part] || part;
    keys.push(name);

    // 组合符号和名称
    if (keySymbolMap[part]) {
      readable.push(`${symbol} ${name}`);
    } else {
      readable.push(part.toUpperCase());
    }
  });

  return {
    symbols,
    readable: `按住 ${readable.join(' + ')}`,
    keys,
  };
}

/**
 * 生成用于显示的快捷键 JSX 元素数据
 */
export function getShortcutDisplayParts(shortcut: string) {
  const parts = shortcut.split('+').map(s => s.trim());

  return parts.map(part => ({
    symbol: keySymbolMap[part] || part.toUpperCase(),
    name: keyNameMap[part] || part,
    hasSymbol: !!keySymbolMap[part],
  }));
}
