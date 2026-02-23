const fs = require('fs')
const path = require('path')

const title = process.argv[2]
if (!title) {
  console.error('Usage: npm run new:deck -- "My Deck Title"')
  process.exit(1)
}

function slugify(str) {
  return str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
}

const slug = slugify(title)
const dest = path.join(__dirname, '..', 'src', 'decks', `${slug}.md`)

if (fs.existsSync(dest)) {
  console.error(`Already exists: src/decks/${slug}.md`)
  process.exit(1)
}

const content = `---
title: ${title}
---

# ${title}

Your first slide.

---

## Slide Two

Your second slide.
`

fs.writeFileSync(dest, content)
console.log(`Created: src/decks/${slug}.md`)
