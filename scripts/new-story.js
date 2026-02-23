const fs = require('fs')
const path = require('path')

const title = process.argv[2]
if (!title) {
  console.error('Usage: npm run new:story -- "My Story Title"')
  process.exit(1)
}

function slugify(str) {
  return str.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
}

const slug = slugify(title)
const storyDir = path.join(__dirname, '..', 'src', 'fiction', slug)

if (fs.existsSync(storyDir)) {
  console.error(`Already exists: src/fiction/${slug}/`)
  process.exit(1)
}

fs.mkdirSync(storyDir, { recursive: true })

const storyMeta = `---
title: ${title}
description:
---
`

const firstChapter = `---
title: Chapter One
order: 1
---

Write your first chapter here.
`

fs.writeFileSync(path.join(storyDir, 'story.md'), storyMeta)
fs.writeFileSync(path.join(storyDir, 'chapter-01.md'), firstChapter)

console.log(`Created: src/fiction/${slug}/story.md`)
console.log(`Created: src/fiction/${slug}/chapter-01.md`)
