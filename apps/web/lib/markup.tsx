// Renders the engine's tiny trace markup (`x_{sub}`, `x^{sup}`) as real sub/superscripts.
// The engine already provides plain-text renderings for calculations; references and titles may still carry markup.
import { Fragment, type ReactNode } from "react";

export function Markup({ text }: { text: string }) {
  const out: ReactNode[] = [];
  let buf = "";
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    if ((c === "_" || c === "^") && text[i + 1] === "{") {
      const end = text.indexOf("}", i + 2);
      if (end > 0) {
        if (buf) { out.push(buf); buf = ""; }
        const inner = text.slice(i + 2, end);
        out.push(c === "_" ? <sub key={i}>{inner}</sub> : <sup key={i}>{inner}</sup>);
        i = end + 1;
        continue;
      }
    }
    buf += c;
    i++;
  }
  if (buf) out.push(buf);
  return <>{out.map((x, k) => <Fragment key={k}>{x}</Fragment>)}</>;
}
