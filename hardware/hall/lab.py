"""PIL label overlay. Orthographic orbit camera; framing calibrated from the render's own alpha bbox
(the membrane walls are the hall's bbox, so projected corners <-> content bbox)."""
import math
from PIL import Image, ImageDraw, ImageFont
F = '/System/Library/Fonts/SFNS.ttf'
def proj(az, el):
    a, e = math.radians(az), math.radians(el)
    r = (-math.sin(a), math.cos(a), 0.0)
    u = (-math.cos(a) * math.sin(e), -math.sin(a) * math.sin(e), math.cos(e))
    return lambda p: (sum(p[i] * r[i] for i in range(3)), sum(p[i] * u[i] for i in range(3)))
def annotate(src, dst, tags, az, el, bbox, fs=26, offs=None, title=None, dims=False, bg=(250, 250, 247), box=None, pad=(0, 0, 0, 0)):
    im0 = Image.open(src).convert('RGBA'); pl, pt, pr, pb = pad
    im = Image.new('RGBA', (im0.width + pl + pr, im0.height + pt + pb), (0, 0, 0, 0)); im.paste(im0, (pl, pt)); W, H = im.size
    ab = box or im.getchannel('A').getbbox()
    P = proj(az, el)
    (x0, y0, z0), (x1, y1, z1) = bbox
    cs = [P((x, y, z)) for x in (x0, x1) for y in (y0, y1) for z in (z0, z1)]
    us, vs = [c[0] for c in cs], [c[1] for c in cs]
    s = ((ab[2] - ab[0]) / (max(us) - min(us)) + (ab[3] - ab[1]) / (max(vs) - min(vs))) / 2
    to = lambda p: (ab[0] + (P(p)[0] - min(us)) * s, ab[1] + (max(vs) - P(p)[1]) * s)
    out = Image.new('RGBA', im.size, bg + (255,)); out.alpha_composite(im)
    d = ImageDraw.Draw(out); f = ImageFont.truetype(F, fs)
    offs = offs or {}
    for lab, x, y, z in tags:
        px, py = to((x, y, z)); dx, dy = offs.get(lab, (40, -40))
        tx, ty = px + dx, py + dy
        tw = d.textlength(lab, font=f)
        d.line([(px, py), (tx, ty)], fill=(20, 20, 30, 255), width=2); d.ellipse([px - 4, py - 4, px + 4, py + 4], fill=(20, 20, 30, 255))
        lx = tx if dx >= 0 else tx - tw
        d.rectangle([lx - 5, ty - fs * 0.65, lx + tw + 5, ty + fs * 0.6], fill=(255, 255, 255, 235), outline=(20, 20, 30, 255))
        d.text((lx, ty), lab, font=f, fill=(15, 15, 25, 255), anchor='lm')
    if dims:
        a, b, c = to((x0, y0, 0)), to((x1, y0, 0)), to((x0, y1, 0))
        yy = a[1] + 30; d.line([(a[0], yy), (b[0], yy)], fill=(20, 20, 30, 255), width=2)
        for xx in (a[0], b[0]): d.line([(xx, yy - 10), (xx, yy + 10)], fill=(20, 20, 30, 255), width=2)
        d.text(((a[0] + b[0]) / 2, yy + 6), '%.1f m' % ((x1 - x0) / 1000), font=f, fill=(15, 15, 25, 255), anchor='mt')
        xx = b[0] + 30; d.line([(xx, a[1]), (xx, c[1])], fill=(20, 20, 30, 255), width=2)
        for q in (a[1], c[1]): d.line([(xx - 10, q), (xx + 10, q)], fill=(20, 20, 30, 255), width=2)
        d.text((xx + 8, (a[1] + c[1]) / 2), '%.1f m' % ((y1 - y0) / 1000), font=f, fill=(15, 15, 25, 255), anchor='lm')
    if title: d.text((30, 25), title, font=ImageFont.truetype(F, int(fs * 1.3)), fill=(15, 15, 25, 255))
    out.convert('RGB').save(dst)
    return to
