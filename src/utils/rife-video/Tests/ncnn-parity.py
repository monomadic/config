#!/usr/bin/env python3
"""Compare native RGB inference with the installed PNG CLI on identical inputs.
PNGs exist only in this reference test's temporary directory, never in rife-video.
Requires numpy and Pillow. Each invocation uses one precision (upstream caches shaders).
"""
import argparse, ctypes as C, json, math, subprocess, tempfile
from pathlib import Path
import numpy as np
from PIL import Image
p = argparse.ArgumentParser()
p.add_argument('library', type=Path)
p.add_argument('input', type=Path)
p.add_argument('--model', type=Path, required=True)
p.add_argument('--reference', default='rife-ncnn-vulkan')
p.add_argument('--fp16', action='store_true')
p.add_argument('--report', type=Path, required=True)
a = p.parse_args()
lib = C.CDLL(str(a.library.resolve()))
lib.rv_ncnn_create.argtypes = [C.c_char_p,C.c_int,C.c_int,C.c_char_p,C.c_size_t]
lib.rv_ncnn_create.restype = C.c_void_p
lib.rv_ncnn_process.argtypes = [C.c_void_p,C.c_void_p,C.c_void_p,C.c_void_p,C.c_int,C.c_int,C.c_float]
lib.rv_ncnn_destroy.argtypes = [C.c_void_p]
err=C.create_string_buffer(1024)
ctx=lib.rv_ncnn_create(str(a.model).encode(),64,int(a.fp16),err,1024)
assert ctx,err.value
probe=json.loads(subprocess.check_output(['ffprobe','-v','error','-select_streams','v:0','-show_entries','stream=width,height','-of','json',str(a.input)]))['streams'][0]
w,h=probe['width'],probe['height']
rows=[]
try:
 with tempfile.TemporaryDirectory(prefix='ncnn-parity-') as temp:
  d=Path(temp)
  for index,phase in [(20,.5),(80,.4),(158,.5)]:
   raw=subprocess.check_output(['ffmpeg','-v','error','-i',str(a.input),'-vf',f'select=between(n\\,{index}\\,{index+2})','-fps_mode','passthrough','-frames:v','3','-pix_fmt','rgb24','-f','rawvideo','pipe:1'])
   frames=np.frombuffer(raw,dtype=np.uint8).reshape(3,h,w,3)
   left,right=frames[0].copy(),frames[2].copy()
   output=np.empty_like(left)
   assert lib.rv_ncnn_process(ctx,left.ctypes.data,right.ctypes.data,output.ctypes.data,w,h,phase)==0
   Image.fromarray(left).save(d/'left.png');Image.fromarray(right).save(d/'right.png')
   result=subprocess.run([a.reference,'-0',str(d/'left.png'),'-1',str(d/'right.png'),'-o',str(d/'out.png'),'-m',str(a.model),'-s',str(phase)],capture_output=True)
   assert result.returncode==0,result.stderr.decode()
   ref=np.array(Image.open(d/'out.png').convert('RGB'))
   delta=output.astype(float)-ref.astype(float)
   mse=float(np.mean(delta**2))
   row=dict(source_index=index,phase=phase,psnr_db=10*math.log10(255**2/max(mse,1e-12)),mae=float(np.mean(abs(delta))),max_error=int(np.max(abs(delta))))
   rows.append(row);print(row,flush=True)
finally: lib.rv_ncnn_destroy(ctx)
a.report.write_text(json.dumps(dict(fp16=a.fp16,model=str(a.model),comparisons=rows),indent=2))
assert min(r['psnr_db'] for r in rows)>45, 'native/reference parity below 45 dB'
