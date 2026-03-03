// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  site: "https://skyfskyf.github.io",
  base: "/grift-site",
  integrations: [
    starlight({
      title: "Grift",
      description:
        "A vau-calculus Lisp interpreter for embedded systems — no_std, no_alloc, no unsafe",
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/skyfskyf/grift",
        },
      ],
      sidebar: [
        {
          label: "Home",
          link: "/",
        },
        {
          label: "Blog",
          link: "/blog/",
        },
        {
          label: "Projects",
          link: "/projects/",
        },
        {
          label: "REPL Demo",
          link: "/demo/",
        },
        {
          label: "Language Guide",
          items: [
            { label: "Getting Started", link: "/language/getting-started/" },
            { label: "Language Reference", link: "/language/reference/" },
            { label: "Specification", link: "/language/spec/" },
          ],
        },
        {
          label: "Internals",
          items: [
            { label: "Architecture", link: "/internals/architecture/" },
            { label: "Contributor Guide", link: "/internals/contributor-guide/" },
          ],
        },
      ],
      customCss: ["./src/styles/custom.css"],
    }),
  ],
});
