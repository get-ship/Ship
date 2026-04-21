/**
 * Markdown renderer for the Jinja macro manifest. Emits a TOC + per-category
 * sections with signature, description, arguments, example, and source link.
 *
 * Byte-stable: calling `renderMarkdown(entries, categories)` twice with the
 * same inputs returns byte-identical output.
 */

export function renderMarkdown(entries, categories) {
  const byCategory = new Map();
  const orderedCategories = [...Object.keys(categories), 'Other'];
  for (const cat of orderedCategories) byCategory.set(cat, []);
  for (const e of entries) {
    const cat = categoryFor(e.name, categories);
    byCategory.get(cat).push(e);
  }

  const lines = [];
  lines.push('# Ship Jinja SDK — macro reference');
  lines.push('');
  lines.push('Auto-generated from `apps/shipd/templates/_ship.html.j2` by `scripts/build-jinja-docs.mjs`. Do not edit by hand — re-run the generator.');
  lines.push('');
  lines.push('Import macros into a view template with:');
  lines.push('');
  lines.push('```jinja');
  lines.push("{% from '_ship.html.j2' import head, signal, show, live %}");
  lines.push('```');
  lines.push('');
  lines.push('## Contents');
  lines.push('');
  for (const cat of orderedCategories) {
    const list = byCategory.get(cat);
    if (!list || list.length === 0) continue;
    lines.push(`- [${cat}](#${slugify(cat)})`);
    for (const e of list) {
      lines.push(`  - [${e.name}](#${slugify(e.name)})`);
    }
  }
  lines.push('');
  for (const cat of orderedCategories) {
    const list = byCategory.get(cat);
    if (!list || list.length === 0) continue;
    lines.push(`## ${cat}`);
    lines.push('');
    for (const e of list) {
      renderEntry(lines, e);
    }
  }
  return lines.join('\n') + '\n';
}

function categoryFor(name, categories) {
  for (const [cat, names] of Object.entries(categories)) {
    if (names.includes(name)) return cat;
  }
  return 'Other';
}

function renderEntry(lines, e) {
  lines.push(`### ${e.name}`);
  lines.push('');
  lines.push('```');
  lines.push(e.signature);
  lines.push('```');
  lines.push('');
  if (!e.documented) {
    lines.push('_Undocumented — no docstring in source._');
    lines.push('');
  } else if (e.description) {
    lines.push(e.description);
    lines.push('');
  }
  if (e.args.length > 0) {
    lines.push('**Arguments:**');
    lines.push('');
    for (const a of e.args) {
      const required = a.required ? '*required*' : `optional, default \`${a.default}\``;
      const desc = a.description ? ` — ${a.description}` : '';
      lines.push(`- \`${a.name}\` (${required})${desc}`);
    }
    lines.push('');
  }
  if (e.example) {
    lines.push('**Example:**');
    lines.push('');
    lines.push('```jinja');
    lines.push(e.example);
    lines.push('```');
    lines.push('');
  }
  lines.push(`_Source: \`${e.source}\`_`);
  lines.push('');
}

function slugify(s) {
  return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
}
