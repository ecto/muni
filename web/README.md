# Municipal Robotics Website

Next.js static site for muni.works with React Three Fiber 3D viewer.

## Development

```bash
# Install dependencies
npm install --legacy-peer-deps

# Start development server
npm run dev
# Open http://localhost:3000

# Build for production
npm run build
# Output in ./out/
```

## Stack

- **Next.js 14** (App Router, static export)
- **React 18** + TypeScript
- **React Three Fiber 8** + drei for 3D viewer
- **Tailwind CSS 4** for styling
- **Phosphor Icons** for icons

## Structure

```
web/
├── src/
│   ├── app/           # Next.js App Router pages
│   │   ├── page.tsx   # Home (/)
│   │   ├── about/     # /about
│   │   ├── products/  # /products
│   │   ├── docs/      # /docs
│   │   ├── investors/ # /investors
│   │   ├── log/       # /log
│   │   ├── success/   # /success
│   │   ├── cancel/    # /cancel
│   │   └── viewer/    # /viewer (3D CAD viewer)
│   ├── components/
│   │   ├── layout/    # Header, NavBar, Footer
│   │   ├── ui/        # Card, ConvertKitForm
│   │   └── viewer/    # ModelViewer, Hotspots, etc.
│   ├── lib/           # Utilities (PostHog)
│   ├── styles/        # globals.css
│   └── types/         # TypeScript declarations
├── public/
│   ├── fonts/         # Berkeley Mono woff2
│   ├── images/        # PNG, JPG assets
│   ├── models/        # GLB, STL 3D models
│   └── docs/          # PDF documents
├── next.config.mjs
├── tailwind.config.ts
└── vercel.json        # Redirects for old URLs
```

## Deployment

Deployed to Vercel with static export. The `vercel.json` file includes redirects for old `.html` URLs.

```bash
# Build and export
npm run build

# Deploy to Vercel
vercel --prod
```

## 3D Viewer

The `/viewer` page is a React Three Fiber application for viewing CAD models:

- Interactive 3D model display with orbit controls
- Component hotspots with labels
- Model selector dropdown
- Component explorer panel
- Inspector with dimensions
- Scale references (banana, astronaut, grogu)
- Wireframe toggle
- Custom grid shader

Models are in GLB format, located in `public/models/`.
