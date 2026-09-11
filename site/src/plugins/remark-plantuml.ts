import { visit } from 'unist-util-visit';
import { renderSvg } from '@vgerbot/plantuml';

// Lazy-init: renderSvg auto-loads WASM on first call (Node.js target auto-inits on require).
// No manual init needed — @vgerbot/plantuml handles it internally.

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&')
    .replace(/</g, '<')
    .replace(/>/g, '>')
    .replace(/"/g, '"');
}

export function remarkPlantuml() {
  return async (tree: any) => {
    const targets: Array<{ node: any; index: number; parent: any }> = [];

    visit(tree, 'code', (node: any, index: any, parent: any) => {
      if (node.lang === 'plantuml' && parent && typeof index === 'number') {
        targets.push({ node, index, parent });
      }
    });

    await Promise.all(
      targets.map(async ({ node, index, parent }) => {
        const source: string = node.value || '';
        let svg = '';
        let errorMsg = '';
        try {
          svg = await renderSvg(source);
          if (!svg) errorMsg = 'Rendering returned empty output (parse failure or unsupported diagram type).';
        } catch (e: any) {
          errorMsg = e?.message || String(e);
        }

        const escapedSource = escapeHtml(source);
        const escapedError = escapeHtml(errorMsg);

        const html = svg
          ? `<div class="plantuml-preview">
  <div class="plantuml-render">${svg}</div>
  <details class="plantuml-source">
    <summary>Source</summary>
    <pre><code class="language-plantuml">${escapedSource}</code></pre>
  </details>
</div>`
          : `<div class="plantuml-preview plantuml-error">
  <div class="plantuml-error-msg">
    <strong>Render Error</strong>
    <pre>${escapedError}</pre>
  </div>
  <details class="plantuml-source">
    <summary>Source</summary>
    <pre><code class="language-plantuml">${escapedSource}</code></pre>
  </details>
</div>`;

        parent.children[index] = { type: 'html', value: html };
      })
    );
  };
}
