import vapoursynth as vs

core = vs.core
video_in = core.std.BlankClip(length=100, width=1920, height=1080, fpsnum=24000, fpsden=1001)
video_out = core.svp2.SmoothFps(video_in, grid=0, pel=1, levels=1, block=True)
