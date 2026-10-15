#!/usr/bin/env python3
"""Sequential, matched hardware-HEVC runs; report startup-inclusive throughput.
Use a long input for sustained measurements. Does not scale or modify the input.
"""
import argparse, json, statistics, subprocess
from pathlib import Path
p=argparse.ArgumentParser()
p.add_argument('binary',type=lambda x:Path(x).resolve())
p.add_argument('input',type=lambda x:Path(x).resolve())
p.add_argument('directory',type=Path,help='new directory for outputs/logs/reports')
p.add_argument('--repeats',type=int,default=3)
p.add_argument('--workers',default='1,2,3,4')
p.add_argument('--fp16',action='store_true',help='also test NCNN FP16 arithmetic')
a=p.parse_args()
workers=[int(x) for x in a.workers.split(',')]
assert a.repeats>0 and all(1<=x<=4 for x in workers)
a.directory.mkdir(parents=True,exist_ok=False)
configs=[('ncnn',w,False) for w in workers]
if a.fp16: configs += [('ncnn',w,True) for w in workers]
configs += [('metal',1,False)]
rows=[]
for repeat in range(a.repeats):
 for backend,w,fp16 in configs:
  name=f'{backend}-w{w}-fp{16 if fp16 else 32}-r{repeat}'
  stats=a.directory/f'{name}.json'
  cmd=[str(a.binary),str(a.input),str(a.directory/f'{name}.mp4'),'--backend',backend,'--workers',str(w),'--tier','hq','--codec','hevc','--stats',str(stats)]
  if fp16: cmd += ['--ncnn-fp16']
  with (a.directory/f'{name}.log').open('w') as log:
   subprocess.run(cmd,stdout=log,stderr=log,check=True)
  row=json.loads(stats.read_text());rows.append(row)
  print(name,round(row['wall_s'],3),'seconds',round(row['output_fps_end_to_end'],2),'output fps',flush=True)
summary=[]
for backend,w,fp16 in configs:
 matches=[r for r in rows if r['backend']==backend and r['workers']==w and r['ncnn_fp16_arithmetic']==fp16]
 summary.append(dict(backend=backend,workers=w,fp16=fp16,median_wall_s=statistics.median(r['wall_s'] for r in matches),median_output_fps=statistics.median(r['output_fps_end_to_end'] for r in matches)))
(a.directory/'summary.json').write_text(json.dumps(dict(summary=summary,runs=rows),indent=2))
