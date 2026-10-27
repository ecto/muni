import subprocess, time, json, sys
sys.path.insert(0, '.')
import gen_hall as G, lab
R = __import__('os').environ.get('VCAD_RENDER', 'vcad-render'); L = __import__('os').environ.get('LOON2VCAD', 'loon2vcad')
d, ymax, ne = G.build()
open('hall.loon', 'w').write(d.loon(ymax=ymax))
t = time.time(); subprocess.run('%s hall.loon > hall_raw.vcad' % L, shell=True, check=True); print('eval %.2fs' % (time.time() - t))
BB = ((0, 0, 0), (G.HX, ymax, G.HH))
def r(out, *a):
    t = time.time(); subprocess.run([R, 'hall_raw.vcad', *a, '-o', out], check=True, stderr=subprocess.DEVNULL); print(out, '%.2fs' % (time.time() - t))
r('raw_top.png', '--view', 'top', '--size', '2000', '--auto-aspect', '--fill', '0.9')
OT = {'row A': (-60, -60), 'row B': (-60, -110), 'row C': (-60, -60), 'dock door (W)': (60, -170), 'dock door (E)': (-60, -170),
      'human 1.75 m': (60, 60), 'carried_bench on tender_12': (-60, 140), 'bench (2 arms)': (-60, 90), 'saw': (20, -70),
      'sheet router': (20, -120), 'mill': (20, -70), 'reflow': (20, -120), 'tray rack': (20, -70), 'winder': (20, -120), 'stock': (20, -70)}
tags = [t for t in d.tags]
# empties: alternate leader directions so they don't collide with the machine labels
k = 0
for i, (l, x, y, z) in enumerate(tags):
    if l.startswith('empty'):
        sgn = 1 if y > 7000 or (6000 > y > 2350) else -1
        tags[i] = (l + ' ' * k, x, y, z); OT[l + ' ' * k] = (40, -70 if (y % 4650) > 2350 else 70); k += 1
lab.annotate('raw_top.png', 'hall_top.png', tags, -90, 90, BB, fs=22, offs=OT, dims=True, pad=(40, 90, 120, 90), title='HALL  36 x 14 x 6 m  ·  3 aisle rows on a spine loop  ·  360 bays')
# other views: cutaway docs with the near walls removed (a low camera otherwise sees only membrane)
BBS = {}
def cutdoc(name, cut):
    dc, _, _ = G.build(cut=cut); BBS[name] = dc.bounds(); open(name + '.loon', 'w').write(dc.loon(ymax=ymax)); subprocess.run('%s %s.loon > %s.vcad' % (L, name, name), shell=True, check=True)
cutdoc('hall_cut_sw', ('wall_s', 'wall_w', 'curtain_w', 'door_head_w'))
cutdoc('hall_cut_w', ('wall_w', 'door_head_w'))
cutdoc('hall_cut_e', ('wall_s', 'wall_e', 'curtain_e', 'door_head_e'))
def r2(doc, out, *a):
    t = time.time(); subprocess.run([R, doc, *a, '-o', out], check=True, stderr=subprocess.DEVNULL); print(out, '%.2fs' % (time.time() - t))
# other views: only a few labels
few = [t for t in tags if t[0].strip() in ('row A', 'row B', 'row C', 'carried_bench on tender_12', 'human 1.75 m', 'bench (2 arms)', 'mill', 'tray rack') or t[0] == 'empty bays (bench slot)']
r2('hall_cut_sw.vcad', 'raw_iso.png', '--azimuth', '215', '--elevation', '30', '--size', '1800', '--auto-aspect', '--fill', '0.95')
lab.annotate('raw_iso.png', 'hall_iso.png', few, 215, 30, BBS['hall_cut_sw'], fs=22, pad=(20, 80, 20, 20), offs={'carried_bench on tender_12': (-40, -120), 'human 1.75 m': (40, -90)}, title='HALL  iso (az 215, el 30)  ·  cutaway: south + west membrane walls omitted')
r2('hall_cut_w.vcad', 'raw_dock.png', '--azimuth', '180', '--elevation', '12', '--size', '1800', '--auto-aspect', '--fill', '0.95')
dk = [t for t in tags if t[0].strip() in ('human 1.75 m', 'row A', 'row B', 'row C')]
lab.annotate('raw_dock.png', 'hall_dock.png', dk + [('light curtain (W dock)', 120, 7000, 2400)], 180, 12, BBS['hall_cut_w'], fs=22, pad=(20, 160, 20, 20), offs={'human 1.75 m': (40, -100), 'row A': (40, -110), 'row B': (40, -140), 'row C': (-40, -110), 'light curtain (W dock)': (60, -200)}, title='HALL  from the west dock (az 180, el 12, ortho)  ·  west membrane wall cut away, light curtain kept')
# carry close-up: focus on the carried bench
t = time.time(); subprocess.run([R, 'hall_cut_e.vcad', '--focus', 'carried_bench', '--azimuth=-30', '--elevation', '28', '--size', '1400', '--fill', '0.3', '-o', 'raw_carry.png'], check=True, stderr=subprocess.DEVNULL); print('carry %.2fs' % (time.time() - t))
from PIL import Image, ImageDraw, ImageFont
im = Image.open('raw_carry.png').convert('RGBA'); o = Image.new('RGBA', (im.width, im.height + 80), (250, 250, 247, 255)); o.alpha_composite(im, (0, 80))
dd = ImageDraw.Draw(o); dd.text((30, 25), 'tender_12 carrying a bench on its lift along the east cross-spine (south wall cut away)', font=ImageFont.truetype(lab.F, 34), fill=(15, 15, 25, 255))
o.convert('RGB').save('hall_carry.png')
