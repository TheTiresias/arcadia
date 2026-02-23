const http = require('http')
const fs = require('fs')
const path = require('path')

const PORT = process.env.PORT || 3000
const DIST = path.join(__dirname, '..', 'dist')

const MIME = {
  '.html': 'text/html',
  '.css':  'text/css',
  '.js':   'text/javascript',
  '.woff': 'font/woff',
  '.woff2':'font/woff2',
  '.ttf':  'font/ttf',
  '.eot':  'application/vnd.ms-fontobject',
  '.svg':  'image/svg+xml',
}

http.createServer((req, res) => {
  let urlPath = req.url.split('?')[0]
  if (urlPath.endsWith('/')) urlPath += 'index.html'

  const filePath = path.join(DIST, urlPath)

  // Prevent path traversal outside dist
  if (!filePath.startsWith(DIST)) {
    res.writeHead(403)
    res.end('Forbidden')
    return
  }

  fs.readFile(filePath, (err, data) => {
    if (err) {
      res.writeHead(404, { 'Content-Type': 'text/plain' })
      res.end('Not found')
      return
    }
    const ext = path.extname(filePath)
    res.writeHead(200, { 'Content-Type': MIME[ext] || 'application/octet-stream' })
    res.end(data)
  })
}).listen(PORT, () => {
  console.log(`Serving dist/ at http://localhost:${PORT}`)
})
