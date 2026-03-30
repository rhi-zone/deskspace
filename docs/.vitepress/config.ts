import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

export default withMermaid(
  defineConfig({
    vite: {
      optimizeDeps: {
        include: ['mermaid'],
      },
    },
    title: 'Deskspace',
    description: 'Unified file workspace server',

    base: '/deskspace/',

    themeConfig: {
      nav: [
        { text: 'Guide', link: '/introduction' },
        { text: 'Architecture', link: '/architecture' },
        { text: 'API', link: '/api' },
        { text: 'rhi', link: 'https://docs.rhi.zone/' },
      ],

      sidebar: [
        {
          text: 'Guide',
          items: [
            { text: 'Introduction', link: '/introduction' },
            { text: 'Getting Started', link: '/getting-started' },
          ],
        },
        {
          text: 'Internals',
          items: [
            { text: 'Architecture', link: '/architecture' },
            { text: 'Projections', link: '/projections' },
            { text: 'API Reference', link: '/api' },
          ],
        },
      ],

      socialLinks: [
        { icon: 'github', link: 'https://github.com/rhi-zone/deskspace' },
      ],

      search: {
        provider: 'local',
      },

      editLink: {
        pattern: 'https://github.com/rhi-zone/deskspace/edit/master/docs/:path',
        text: 'Edit this page on GitHub',
      },
    },
  }),
)
