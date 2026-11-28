// Line icons drawn on a 20-unit grid, 1.5 stroke, square caps — matched to the hairline visual language.
import type { SVGProps } from "react";

const base = (p: SVGProps<SVGSVGElement>) => ({ width: 18, height: 18, viewBox: "0 0 20 20", fill: "none", stroke: "currentColor", strokeWidth: 1.5, strokeLinecap: "square" as const, strokeLinejoin: "miter" as const, "aria-hidden": true, ...p });

export const IconWorkbench = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><rect x="2.5" y="3.5" width="15" height="13" /><path d="M7.5 3.5v13M7.5 10h10" /></svg>);
export const IconProject = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><rect x="2.5" y="3" width="5" height="4" /><rect x="12.5" y="8" width="5" height="4" /><rect x="2.5" y="13" width="5" height="4" /><path d="M7.5 5h2.5v10H7.5M10 10h2.5" /></svg>);
export const IconStandards = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M4.5 2.5h9l2 2v13h-11z" /><path d="M7 7h6M7 10h6M7 13h4" /></svg>);
export const IconAccount = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><circle cx="10" cy="7" r="3.5" /><path d="M3.5 17.5c1-3.5 3.5-5 6.5-5s5.5 1.5 6.5 5" /></svg>);
export const IconSun = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><circle cx="10" cy="10" r="3.5" /><path d="M10 2v2M10 16v2M2 10h2M16 10h2M4.3 4.3l1.4 1.4M14.3 14.3l1.4 1.4M4.3 15.7l1.4-1.4M14.3 5.7l1.4-1.4" /></svg>);
export const IconMoon = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M15.5 12.5A6.5 6.5 0 0 1 7.5 4.5a6.5 6.5 0 1 0 8 8z" /></svg>);
export const IconSearch = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><circle cx="8.5" cy="8.5" r="5" /><path d="M12.5 12.5l4.5 4.5" /></svg>);
export const IconPanelLeft = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><rect x="2.5" y="3.5" width="15" height="13" /><path d="M7.5 3.5v13" /></svg>);
export const IconPanelRight = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><rect x="2.5" y="3.5" width="15" height="13" /><path d="M12.5 3.5v13" /></svg>);
export const IconExport = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M10 3v9M6.5 8.5L10 12l3.5-3.5M3.5 13.5v3h13v-3" /></svg>);
export const IconReset = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M4 10a6 6 0 1 0 2-4.5" /><path d="M4 3v3.5h3.5" /></svg>);
export const IconSpark = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M10 2.5v4M10 13.5v4M2.5 10h4M13.5 10h4M5 5l2.5 2.5M12.5 12.5L15 15M5 15l2.5-2.5M12.5 7.5L15 5" /></svg>);
export const IconFit = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M3 7V3h4M13 3h4v4M17 13v4h-4M7 17H3v-4" /></svg>);
export const IconPlus = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M10 4v12M4 10h12" /></svg>);
export const IconMinus = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M4 10h12" /></svg>);
export const IconClose = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M5 5l10 10M15 5L5 15" /></svg>);
export const IconArrow = (p: SVGProps<SVGSVGElement>) => (<svg {...base(p)}><path d="M3.5 10h12M11 5.5l4.5 4.5-4.5 4.5" /></svg>);
