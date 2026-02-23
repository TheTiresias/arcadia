const fs = require('fs')
const path = require('path')

const title = process.argv[2]
if (!title) {
  console.error('Usage: npm run new:post -- "My Post Title"')
  process.exit(1)
}

function slugify(str) {
  return str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
}

const slug = slugify(title)
const date = new Date().toISOString().slice(0, 10)
const dest = path.join(__dirname, '..', 'src', 'posts', `${slug}.md`)

if (fs.existsSync(dest)) {
  console.error(`Already exists: src/posts/${slug}.md`)
  process.exit(1)
}

const content = `---
title: ${title}
date: ${date}
subtitle:
---

Write your post here.
`

fs.writeFileSync(dest, content)
console.log(`Created: src/posts/${slug}.md`)
