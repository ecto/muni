"""Derive the render docs for the bay mockup from bay.vcad / bay_one_raw.vcad (recovered from the transcript's ad-hoc steps).
bay_empty.vcad   : bay.vcad minus the bench (bay_names.txt instances only, frame shrunk to a 1 mm tag)
bay_faceonly.vcad: bay_empty minus plug / pin blocks / tender (clear view of the coupler face)
bay_one.vcad     : bay 1 + 900 mm of spine only (detail iso)"""
import json
def sub(src, dst, keep, drop=lambda i: False):
    d = json.load(open(src))
    d['instances'] = [i for i in d['instances'] if (i['id'] in keep or i['id'] == 'frame') and not drop(i['id'])]
    d['joints'] = [j for j in d['joints'] if j['childInstanceId'] in keep and not drop(j['childInstanceId'])]
    nid = 99999; d['nodes'][str(nid)] = {'id': nid, 'name': None, 'op': {'type': 'Cube', 'size': {'x': 1.0, 'y': 1.0, 'z': 1.0}}}
    d['partDefs']['frame']['root'] = nid
    json.dump(d, open(dst, 'w'))
names = set(open('bay_names.txt').read().split())
sub('bay.vcad', 'bay_empty.vcad', names - {'bench_plate', 'bench_plug', 'bench_plug_hanger'})
sub('bay.vcad', 'bay_faceonly.vcad', names - {'bench_plate', 'bench_plug', 'bench_plug_hanger'}, lambda i: i.startswith(('plug', 'blk', 'tender')))
sub('bay_one_raw.vcad', 'bay_one.vcad', names)
