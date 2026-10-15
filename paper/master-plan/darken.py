import glob, os
import numpy as np
from PIL import Image
# palette ramp: luminance-inverted value t in [0,1] -> bg .. line_hi .. label .. text
stops=[(0.0,(0x0b,0x09,0x07)),(0.25,(0x4a,0x3a,0x2a)),(0.6,(0xa0,0x8d,0x70)),(1.0,(0xe8,0xe1,0xd2))]
def ramp(t):
    out=np.zeros(t.shape+(3,))
    for (a,ca),(b,cb) in zip(stops,stops[1:]):
        m=(t>=a)&(t<=b); f=((t-a)/(b-a))[m][:,None]
        out[m]=np.array(ca)*(1-f)+np.array(cb)*f
    return out
for f in sorted(glob.glob('images/*.png')):
    im=Image.open(f).convert('RGBA')
    bg=Image.new('RGBA',im.size,(255,255,255,255)); bg.alpha_composite(im)
    hall=os.path.basename(f).startswith('hall_grow_')
    if hall: bg=bg.crop((0,60,1940,1065))  # drop baked title/footer text
    a=np.asarray(bg.convert('RGB')).astype(float)/255
    L=0.2126*a[...,0]+0.7152*a[...,1]+0.0722*a[...,2]
    Lb=np.median(np.concatenate([L[:4].ravel(),L[-4:].ravel(),L[:,:4].ravel(),L[:,-4:].ravel()]))
    t=np.clip((Lb-L)/max(Lb,1e-3),0,1)
    t=np.clip(t*1.2,0,1)**1.35
    if hall:  # empty bays (green 'empty' material) read as holes
        c=a*255; t[(c[...,1]>c[...,0]+60)&(c[...,1]>c[...,2]+40)]=0
    rgb=ramp(t).astype(np.uint8)
    out=Image.fromarray(rgb)
    # trim
    mask=(t>0.08); ys,xs=np.where(mask)
    if len(xs):
        pad=int(0.03*max(out.size))
        box=(max(xs.min()-pad,0),max(ys.min()-pad,0),min(xs.max()+pad,out.size[0]),min(ys.max()+pad,out.size[1]))
        out=out.crop(box)
    out.save('images-dark/'+os.path.basename(f))
    print(f,out.size)
