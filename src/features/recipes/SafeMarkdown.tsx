import type { ReactNode } from 'react';

// Deliberately small Markdown subset. All text remains React text nodes: raw HTML,
// images, embedded media and URL protocols are never interpreted or requested.
function inline(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const pattern = /(`[^`\n]+`|\*\*[^*\n]+\*\*|\*[^*\n]+\*|\[[^\]\n]+\]\([^\s)]+\))/g;
  let cursor = 0;
  for (const match of text.matchAll(pattern)) {
    const index = match.index ?? 0;
    if (index > cursor) nodes.push(text.slice(cursor, index));
    const token = match[0];
    const key = `${index}-${nodes.length}`;
    if (token.startsWith('`')) nodes.push(<code key={key}>{token.slice(1, -1)}</code>);
    else if (token.startsWith('**')) nodes.push(<strong key={key}>{token.slice(2, -2)}</strong>);
    else if (token.startsWith('*')) nodes.push(<em key={key}>{token.slice(1, -1)}</em>);
    else {
      const end = token.indexOf('](');
      const label = token.slice(1, end);
      const raw = token.slice(end + 2, -1);
      try {
        const url = new URL(raw);
        if (url.protocol === 'https:' || url.protocol === 'http:') {
          nodes.push(<a key={key} href={url.href} target="_blank" rel="noopener noreferrer">{label}</a>);
        } else nodes.push(label);
      } catch { nodes.push(label); }
    }
    cursor = index + token.length;
  }
  if (cursor < text.length) nodes.push(text.slice(cursor));
  return nodes;
}

export function SafeMarkdown({source}:{source:string}) {
  return <div className="safe-markdown">{source.split(/\r?\n/).map((line, index) => {
    const key = index;
    if (line.startsWith('### ')) return <h4 key={key}>{inline(line.slice(4))}</h4>;
    if (line.startsWith('## ')) return <h3 key={key}>{inline(line.slice(3))}</h3>;
    if (line.startsWith('# ')) return <h2 key={key}>{inline(line.slice(2))}</h2>;
    if (/^[-*] /.test(line)) return <p key={key}>• {inline(line.slice(2))}</p>;
    return <p key={key}>{inline(line)}</p>;
  })}</div>;
}
