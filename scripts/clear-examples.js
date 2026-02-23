const fs = require('fs')
const path = require('path')

const examples = [
  path.join(__dirname, '..', 'src', 'posts', 'hello-world.md'),
  path.join(__dirname, '..', 'src', 'fiction', 'example-story'),
  path.join(__dirname, '..', 'src', 'decks', 'example-deck.md'),
]

let removed = 0

for (const target of examples) {
  if (!fs.existsSync(target)) continue
  fs.rmSync(target, { recursive: true, force: true })
  const rel = target.replace(path.join(__dirname, '..') + path.sep, '')
  console.log(`Removed: ${rel}`)
  removed++
}

if (removed === 0) {
  console.log('No example content found — already cleared.')
} else {
  console.log(`\nDone. Run npm run build to rebuild the site.`)
}
