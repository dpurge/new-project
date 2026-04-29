import { defineConfig } from "@solidjs/start/config";
import { withSolidBase } from "@kobalte/solidbase/config";

export default defineConfig(
  withSolidBase(
    {
      ssr: true,
    },
    {
      title: "{{ cookiecutter.site_title }}",
      titleTemplate: "%s | {{ cookiecutter.site_title }}",
      description: "{{ cookiecutter.description }}",
      lastUpdated: false,
      editPath:
        "https://github.com/your-org/{{ cookiecutter.project_slug }}/edit/main/src/routes/:path",
      themeConfig: {
        nav: [
          {
            text: "Guide",
            link: "/guide/getting-started",
          },
          {
            text: "Reference",
            link: "/reference/project-structure",
          },
        ],
        sidebar: {
          "/guide": [
            {
              title: "Guide",
              collapsed: false,
              items: [
                {
                  title: "Overview",
                  link: "/",
                },
                {
                  title: "Getting Started",
                  link: "/guide/getting-started",
                  status: "new",
                },
              ],
            },
          ],
          "/reference": [
            {
              title: "Reference",
              collapsed: false,
              items: [
                {
                  title: "Project Structure",
                  link: "/reference/project-structure",
                },
              ],
            },
          ],
        },
        socialLinks: {
          github: "https://github.com/your-org/{{ cookiecutter.project_slug }}",
        },
        footer: {
          message: "Built with Deno, SolidStart, SolidBase, and MDX.",
          copyright: "Copyright (c) {{ cookiecutter.project_name }}",
        },
      },
      markdown: {
        toc: {
          minDepth: 2,
          maxDepth: 4,
        },
      },
    },
  ),
);
