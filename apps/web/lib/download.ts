// Browser file downloads.
export function downloadText(filename: string, text: string, mime: string) {
  const url = URL.createObjectURL(new Blob([text], { type: mime }));
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** Opens an HTML document in a new tab and starts the print dialog. Returns false if pop-ups are blocked. */
export function printHtml(html: string): boolean {
  const w = window.open("", "_blank");
  if (!w) return false;
  w.document.write(html);
  w.document.close();
  let printed = false;
  const go = () => {
    if (printed) return;
    printed = true;
    try { w.focus(); w.print(); } catch { /* window was closed */ }
  };
  w.addEventListener("load", go);
  setTimeout(go, 600);
  return true;
}
