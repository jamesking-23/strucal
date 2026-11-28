// STATIC_EXPORT=1 produces a fully static site in out/ (used for GitHub Pages); NEXT_PUBLIC_BASE_PATH sets the
// sub-path it is served from (e.g. /strucal). Without them, `next dev` / `next start` behave as usual.
const staticExport = process.env.STATIC_EXPORT === "1";
const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "";

const nextConfig = {
  reactStrictMode: true,
  ...(staticExport ? { output: "export", trailingSlash: true, images: { unoptimized: true } } : {}),
  ...(basePath ? { basePath } : {}),
};
export default nextConfig;
