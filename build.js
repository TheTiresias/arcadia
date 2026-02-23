const fs = require('fs')
const path = require('path')
const matter = require('gray-matter')
const MarkdownIt = require('markdown-it')

const md = new MarkdownIt({ html: true, typographer: true })

// Sidenote plugin: ^[text] → tufte numbered sidenote
// Margin note plugin: >[text] → tufte unnumbered margin note
md.inline.ruler.push('tufte_sidenote', function (state, silent) {
  const src = state.src
  const pos = state.pos

  // ^[ ... ] for sidenotes
  if (src[pos] === '^' && src[pos + 1] === '[') {
    const end = findClosingBracket(src, pos + 2, state.posMax)
    if (end === -1) return false
    if (!silent) {
      const token = state.push('sidenote', '', 0)
      token.content = src.slice(pos + 2, end)
    }
    state.pos = end + 1
    return true
  }

  // >[ ... ] for margin notes
  if (src[pos] === '>' && src[pos + 1] === '[') {
    const end = findClosingBracket(src, pos + 2, state.posMax)
    if (end === -1) return false
    if (!silent) {
      const token = state.push('marginnote', '', 0)
      token.content = src.slice(pos + 2, end)
    }
    state.pos = end + 1
    return true
  }

  return false
})

function findClosingBracket(src, from, max) {
  let depth = 1
  let pos = from
  while (pos <= max) {
    if (src[pos] === '[') depth++
    if (src[pos] === ']') { if (--depth === 0) return pos }
    pos++
  }
  return -1
}

md.renderer.rules['sidenote'] = function (tokens, idx, options, env) {
  env.noteCount = (env.noteCount || 0) + 1
  const id = `sn-${env.noteCount}`
  const content = tokens[idx].content
  return (
    `<label for="${id}" class="margin-toggle sidenote-number"></label>` +
    `<input type="checkbox" id="${id}" class="margin-toggle"/>` +
    `<span class="sidenote">${content}</span>`
  )
}

md.renderer.rules['marginnote'] = function (tokens, idx, options, env) {
  env.noteCount = (env.noteCount || 0) + 1
  const id = `mn-${env.noteCount}`
  const content = tokens[idx].content
  return (
    `<label for="${id}" class="margin-toggle">&#8853;</label>` +
    `<input type="checkbox" id="${id}" class="margin-toggle"/>` +
    `<span class="marginnote">${content}</span>`
  )
}

// Paths
const ROOT = __dirname
const DIST = path.join(ROOT, 'dist')
const SRC_POSTS = path.join(ROOT, 'src', 'posts')
const TEMPLATES = path.join(ROOT, 'templates')

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true })
}

function readTemplate(name) {
  return fs.readFileSync(path.join(TEMPLATES, name), 'utf8')
}

function formatDate(raw) {
  if (!raw) return ''
  if (raw instanceof Date) return raw.toISOString().slice(0, 10)
  return String(raw)
}

// Wrap content in <section> blocks, splitting on <hr>
function sectionWrap(html) {
  const parts = html.split(/<hr\s*\/?>/)
  return parts.map(p => `<section>\n${p.trim()}\n</section>`).join('\n')
}

function buildPosts() {
  ensureDir(path.join(DIST, 'posts'))
  const template = readTemplate('post.html')
  const files = fs.readdirSync(SRC_POSTS).filter(f => f.endsWith('.md'))
  const posts = []

  for (const file of files) {
    const raw = fs.readFileSync(path.join(SRC_POSTS, file), 'utf8')
    const { data, content } = matter(raw)
    const env = {}
    const bodyHtml = sectionWrap(md.render(content, env))
    const slug = file.replace(/\.md$/, '')
    const date = formatDate(data.date)

    const rendered = template
      .replace(/\{\{title\}\}/g, data.title || slug)
      .replace('{{subtitle}}', data.subtitle ? `<p class="subtitle">${data.subtitle}</p>` : '')
      .replace('{{date}}', date ? `<p class="date">${date}</p>` : '')
      .replace('{{content}}', bodyHtml)
      .replace(/\{\{root\}\}/g, '..')

    fs.writeFileSync(path.join(DIST, 'posts', `${slug}.html`), rendered)
    posts.push({ slug, title: data.title || slug, date })
    console.log(`  built posts/${slug}.html`)
  }

  return posts
}

function buildFiction() {
  const fictionSrc = path.join(ROOT, 'src', 'fiction')
  if (!fs.existsSync(fictionSrc)) return []

  ensureDir(path.join(DIST, 'fiction'))

  const storyDirs = fs.readdirSync(fictionSrc).filter(f =>
    fs.statSync(path.join(fictionSrc, f)).isDirectory()
  )

  const chapterTemplate = readTemplate('chapter.html')
  const tocTemplate = readTemplate('story-toc.html')
  const stories = []

  for (const storySlug of storyDirs) {
    const storyDir = path.join(fictionSrc, storySlug)
    ensureDir(path.join(DIST, 'fiction', storySlug))

    // Story metadata
    const storyMetaPath = path.join(storyDir, 'story.md')
    const storyMeta = fs.existsSync(storyMetaPath)
      ? matter(fs.readFileSync(storyMetaPath, 'utf8')).data
      : { title: storySlug }

    const bodyStyle = (storyMeta.background_color || storyMeta.font_color)
      ? ` style="${storyMeta.background_color ? `background-color:${storyMeta.background_color};` : ''}${storyMeta.font_color ? `color:${storyMeta.font_color};` : ''}"`
      : ''

    // Load and sort chapters by `order` frontmatter
    const chapterFiles = fs.readdirSync(storyDir)
      .filter(f => f.endsWith('.md') && f !== 'story.md')

    const chapters = chapterFiles.map(file => {
      const raw = fs.readFileSync(path.join(storyDir, file), 'utf8')
      const { data, content } = matter(raw)
      return { file, slug: file.replace(/\.md$/, ''), data, content }
    }).sort((a, b) => (a.data.order || 0) - (b.data.order || 0))

    // Build each chapter page
    for (let i = 0; i < chapters.length; i++) {
      const chapter = chapters[i]
      const prev = i > 0 ? chapters[i - 1] : null
      const next = i < chapters.length - 1 ? chapters[i + 1] : null

      const prevLink = prev
        ? `<a href="${prev.slug}.html">← ${prev.data.title || 'Previous'}</a>`
        : `<span class="inactive">←</span>`
      const nextLink = next
        ? `<a href="${next.slug}.html">${next.data.title || 'Next'} →</a>`
        : `<span class="inactive">→</span>`
      const contentsLink = `<a href="index.html">Contents</a>`
      const nav = `<nav class="chapter-nav">${prevLink} — ${contentsLink} — ${nextLink}</nav>`

      const env = {}
      const bodyHtml = sectionWrap(md.render(chapter.content, env))

      const rendered = chapterTemplate
        .replace(/\{\{title\}\}/g, chapter.data.title || chapter.slug)
        .replace('{{subtitle}}', chapter.data.subtitle ? `<p class="subtitle">${chapter.data.subtitle}</p>` : '')
        .replace(/\{\{nav\}\}/g, nav)
        .replace('{{content}}', bodyHtml)
        .replace(/\{\{story_title\}\}/g, storyMeta.title || storySlug)
        .replace('{{body_style}}', bodyStyle)
        .replace(/\{\{root\}\}/g, '../..')

      fs.writeFileSync(path.join(DIST, 'fiction', storySlug, `${chapter.slug}.html`), rendered)
      console.log(`  built fiction/${storySlug}/${chapter.slug}.html`)
    }

    // Build story ToC
    const items = chapters.map(c =>
      `      <li><a href="${c.slug}.html">${c.data.title || c.slug}</a></li>`
    ).join('\n')

    const tocRendered = tocTemplate
      .replace(/\{\{title\}\}/g, storyMeta.title || storySlug)
      .replace('{{description}}', storyMeta.description ? `<p>${storyMeta.description}</p>` : '')
      .replace('{{chapters}}', items)
      .replace('{{body_style}}', bodyStyle)
      .replace(/\{\{root\}\}/g, '../..')

    fs.writeFileSync(path.join(DIST, 'fiction', storySlug, 'index.html'), tocRendered)
    console.log(`  built fiction/${storySlug}/index.html`)

    stories.push({
      slug: storySlug,
      title: storyMeta.title || storySlug,
      description: storyMeta.description || '',
      chapterCount: chapters.length
    })
  }

  // Build top-level fiction.html
  const fictionIndexTemplate = readTemplate('fiction-index.html')
  const storyItems = stories.map(s =>
    `      <li><a href="fiction/${s.slug}/index.html">${s.title}</a>` +
    (s.description ? ` — ${s.description}` : '') +
    ` <span class="date">(${s.chapterCount} chapter${s.chapterCount !== 1 ? 's' : ''})</span></li>`
  ).join('\n')

  const fictionHtml = fictionIndexTemplate
    .replace('{{stories}}', storyItems)
    .replace(/\{\{root\}\}/g, '.')

  fs.writeFileSync(path.join(DIST, 'fiction.html'), fictionHtml)
  console.log('  built fiction.html')

  return stories
}

function buildDecks() {
  const decksSrc = path.join(ROOT, 'src', 'decks')
  if (!fs.existsSync(decksSrc)) return []

  ensureDir(path.join(DIST, 'decks'))

  const template = readTemplate('slide-deck.html')
  const files = fs.readdirSync(decksSrc).filter(f => f.endsWith('.md'))
  const decks = []

  for (const file of files) {
    const raw = fs.readFileSync(path.join(decksSrc, file), 'utf8')
    const { data, content } = matter(raw)
    const slug = file.replace(/\.md$/, '')

    const slideContents = content.split(/\n---\n/)
    const slidesHtml = slideContents
      .map(s => s.trim())
      .filter(s => s.length > 0)
      .map(s => `<div class="slide">\n${md.render(s, {})}\n</div>`)
      .join('\n')

    const bodyStyle = (data.background_color || data.font_color)
      ? ` style="${data.background_color ? `background-color:${data.background_color};` : ''}${data.font_color ? `color:${data.font_color};` : ''}"`
      : ''

    const rendered = template
      .replace(/\{\{title\}\}/g, data.title || slug)
      .replace('{{slides}}', slidesHtml)
      .replace('{{body_style}}', bodyStyle)
      .replace(/\{\{root\}\}/g, '..')

    fs.writeFileSync(path.join(DIST, 'decks', `${slug}.html`), rendered)
    console.log(`  built decks/${slug}.html`)
    decks.push({ slug, title: data.title || slug })
  }

  // Build decks index
  const indexTemplate = readTemplate('decks-index.html')
  const items = decks.map(d =>
    `      <li><a href="decks/${d.slug}.html">${d.title}</a></li>`
  ).join('\n')

  const html = indexTemplate
    .replace('{{decks}}', items)
    .replace(/\{\{root\}\}/g, '.')

  fs.writeFileSync(path.join(DIST, 'decks.html'), html)
  console.log('  built decks.html')

  return decks
}

function buildIndex(posts) {
  const template = readTemplate('index.html')
  const sorted = posts.slice().sort((a, b) => (b.date > a.date ? 1 : -1))
  const items = sorted.map(p =>
    `      <li><a href="posts/${p.slug}.html">${p.title}</a>` +
    (p.date ? ` <span class="date">${p.date}</span>` : '') +
    `</li>`
  ).join('\n')

  const html = template
    .replace('{{posts}}', items)
    .replace(/\{\{root\}\}/g, '.')

  fs.writeFileSync(path.join(DIST, 'index.html'), html)
  console.log('  built index.html')
}

function copyAssets() {
  const tufteDir = path.join(ROOT, 'node_modules', 'tufte-css')
  fs.copyFileSync(path.join(tufteDir, 'tufte.css'), path.join(DIST, 'tufte.css'))
  fs.cpSync(path.join(tufteDir, 'et-book'), path.join(DIST, 'et-book'), { recursive: true })
  console.log('  copied tufte.css + et-book fonts')
}

console.log('Building...')
ensureDir(DIST)
copyAssets()
const posts = buildPosts()
buildFiction()
buildDecks()
buildIndex(posts)
console.log('Done.')
