# Muni Website

Static website for [muni.works](https://muni.works), hosted on GitHub Pages.

## Adding Media Files

**Important:** GitHub Pages does not support Git LFS. The repository uses LFS for large binary files, but web assets are excluded.

When adding images or other media to the website:

1. **Copy files to `web/`** (do not symlink from other directories)
2. **Verify the file is not tracked by LFS:**
   ```bash
   git check-attr filter web/your-image.jpg
   # Should output: web/your-image.jpg: filter: unset
   ```
3. **If the file shows `filter: lfs`**, the `.gitattributes` rules need updating

### Supported formats (excluded from LFS)

These patterns in `.gitattributes` exclude web assets from LFS:

```
web/*.png -filter -diff -merge binary
web/*.jpg -filter -diff -merge binary
web/*.pdf -filter -diff -merge binary
web/**/*.pdf -filter -diff -merge binary
```

### Adding new file types

If you need to add a new media type (e.g., `.webp`, `.mp4`), add an exclusion rule to `.gitattributes`:

```
web/*.webp -filter -diff -merge binary
```

**Rule order matters:** exclusions must come AFTER the general LFS rules in the file.

### Cache busting

After migrating files out of LFS, browsers may cache the old (broken) version. Add a query param to force a refresh:

```html
<img src="image.jpg?v=2" alt="...">
```

## Local Development

Open `index.html` in a browser, or use a local server:

```bash
python3 -m http.server 8000
# Then visit http://localhost:8000
```

## Deployment

Push to `main` branch. GitHub Pages automatically deploys from the `web/` folder.
