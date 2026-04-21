#!/usr/bin/env python3
"""
davinci-resolve-interpolate-60fps.py

Import one input clip into DaVinci Resolve, apply Optical Flow interpolation,
and render to HEVC MP4. Attempts to make the project/timeline 60 fps as early
as possible, but tolerates Resolve builds that lock frame rate unexpectedly.

Usage:
  davinci-resolve-interpolate-60fps.py /absolute/input.mp4 /absolute/output.mp4
  davinci-resolve-interpolate-60fps.py --speed-warp /absolute/input.mp4 /absolute/output.mp4
  davinci-resolve-interpolate-60fps.py --debug-formats /absolute/input.mp4 /absolute/output.mp4
"""

from __future__ import annotations

import argparse
import sys
import time
import uuid
from pathlib import Path


def fail(msg: str, code: int = 1) -> None:
    print(f"error: {msg}", file=sys.stderr)
    raise SystemExit(code)


def warn(msg: str) -> None:
    print(f"warning: {msg}", file=sys.stderr)


def require_absolute_path(p: str) -> Path:
    q = Path(p).expanduser().resolve()
    if not q.is_absolute():
        fail(f"path must be absolute: {q}")
    return q


def load_resolve_module():
    try:
        import DaVinciResolveScript as dvr  # type: ignore
        return dvr
    except ImportError as exc:
        fail(
            "could not import DaVinciResolveScript. "
            "Set RESOLVE_SCRIPT_API / RESOLVE_SCRIPT_LIB / PYTHONPATH first."
        )
        raise exc


def wait_for_render(project, poll_s: float = 1.0) -> None:
    while project.IsRenderingInProgress():
        time.sleep(poll_s)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render one input clip through DaVinci Resolve as HEVC 60fps MP4."
    )
    parser.add_argument(
        "--speed-warp",
        action="store_true",
        help="Use Speed Warp motion estimation instead of Enhanced Better",
    )
    parser.add_argument(
        "--debug-formats",
        action="store_true",
        help="Print available render formats/codecs before rendering",
    )
    parser.add_argument("input_file", help="Absolute input file path")
    parser.add_argument("output_file", help="Absolute output file path")
    return parser.parse_args()


def find_format_token(project, wanted_token: str) -> str:
    formats = project.GetRenderFormats() or {}
    for _desc, token in formats.items():
        if str(token).lower() == wanted_token.lower():
            return str(token)
    fail(f"render format unavailable: {wanted_token}; available formats: {formats}")


def find_hevc_codec_token(project, format_token: str) -> str:
    codecs = project.GetRenderCodecs(format_token) or {}
    for desc, token in codecs.items():
        t = str(token).lower()
        d = str(desc).lower()
        if "265" in t or "hevc" in t or "265" in d or "hevc" in d:
            return str(token)
    fail(f"could not find H.265/HEVC codec in {format_token} codecs: {codecs}")


def debug_render_formats(project) -> None:
    formats = project.GetRenderFormats() or {}
    print("available render formats:")
    for desc, token in formats.items():
        print(f"  {desc}: {token}")

    for desc, token in formats.items():
        codecs = project.GetRenderCodecs(str(token)) or {}
        if codecs:
            print(f"\ncodecs for {desc} ({token}):")
            for codec_desc, codec_token in codecs.items():
                print(f"  {codec_desc}: {codec_token}")


def try_set_project_fps(project, fps: str) -> bool:
    """
    Resolve may reject this depending on project state/build.
    Must be attempted before import/timeline creation.
    """
    ok = False
    try:
        ok = bool(project.SetSetting("timelineFrameRate", fps))
    except Exception:
        ok = False

    if not ok:
        return False

    # Optional extras; ignore failures.
    for key in ("timelinePlaybackFrameRate",):
        try:
            project.SetSetting(key, fps)
        except Exception:
            pass

    return True


def try_set_timeline_fps(timeline, fps: str) -> bool:
    """
    Some builds expose timeline-level setters; some do not.
    """
    ok_any = False
    for key in ("timelineFrameRate", "timelinePlaybackFrameRate"):
        try:
            ok = bool(timeline.SetSetting(key, fps))
            ok_any = ok_any or ok
        except Exception:
            pass
    return ok_any


def ensure_empty_dir_exists(dir_path: Path) -> None:
    if not dir_path.exists():
        dir_path.mkdir(parents=True, exist_ok=True)
    if not dir_path.is_dir():
        fail(f"output directory is not a directory: {dir_path}")


def main() -> None:
    args = parse_args()

    input_file = require_absolute_path(args.input_file)
    output_file = require_absolute_path(args.output_file)

    if not input_file.exists():
        fail(f"input file not found: {input_file}")

    output_dir = output_file.parent
    output_name = output_file.stem
    ensure_empty_dir_exists(output_dir)

    dvr = load_resolve_module()
    resolve = dvr.scriptapp("Resolve")
    if resolve is None:
        fail("could not connect to Resolve")

    project_manager = resolve.GetProjectManager()
    if project_manager is None:
        fail("could not get ProjectManager")

    project_name = f"auto_interp_{uuid.uuid4().hex[:12]}"
    project = project_manager.CreateProject(project_name)
    if project is None:
        fail(f"could not create project: {project_name}")

    try:
        resolve.OpenPage("edit")

        if args.debug_formats:
            debug_render_formats(project)

        # Critical: try to set project FPS before importing media or creating a timeline.
        project_fps_set = try_set_project_fps(project, "60")
        if not project_fps_set:
            warn("could not set project timelineFrameRate=60 before import; continuing")

        media_storage = resolve.GetMediaStorage()
        media_pool = project.GetMediaPool()
        if media_storage is None or media_pool is None:
            fail("could not access media storage or media pool")

        imported = media_storage.AddItemListToMediaPool([str(input_file)])
        if not imported:
            fail("failed to import input media")

        media_item = imported[0]

        # Create timeline from clip. On many Resolve builds this will inherit the locked/default project FPS.
        timeline_name = f"tl_{input_file.stem}"
        timeline = media_pool.CreateTimelineFromClips(timeline_name, [media_item])
        if timeline is None:
            fail("failed to create timeline")

        project.SetCurrentTimeline(timeline)

        timeline_fps_set = try_set_timeline_fps(timeline, "60")
        if not timeline_fps_set and not project_fps_set:
            warn("could not explicitly force timeline FPS to 60; render may still succeed at 60 if allowed by your build")

        items = timeline.GetItemListInTrack("video", 1)
        if not items:
            fail("no timeline items found on video track 1")

        clip = items[0]

        # Resolve scripting values commonly documented for these properties:
        # RetimeProcess: 3 => Optical Flow
        # MotionEstimation: 4 => Enhanced Better, 5 => Speed Warp
        RETIME_OPTICAL_FLOW = 3
        MOTION_EST_ENHANCED_BETTER = 4
        MOTION_EST_SPEED_WARP = 5

        motion_estimation = (
            MOTION_EST_SPEED_WARP if args.speed_warp else MOTION_EST_ENHANCED_BETTER
        )

        ok = True
        ok &= bool(clip.SetProperty("RetimeProcess", RETIME_OPTICAL_FLOW))
        ok &= bool(clip.SetProperty("MotionEstimation", motion_estimation))
        if not ok:
            fail("failed to set clip interpolation properties")

        project.DeleteAllRenderJobs()

        format_token = find_format_token(project, "mp4")
        codec_token = find_hevc_codec_token(project, format_token)

        if not project.SetCurrentRenderFormatAndCodec(format_token, codec_token):
            fail(f"failed to set render format/codec: {format_token}/{codec_token}")

        # Keep render settings minimal first.
        render_settings = {
            "SelectAllFrames": True,
            "TargetDir": str(output_dir),
            "CustomName": output_name,
            "ExportVideo": True,
            "ExportAudio": True,
        }

        # Some builds accept render frame rate as an explicit deliver setting, some reject it.
        # Try it, then fall back silently.
        try_with_fps = dict(render_settings)
        try_with_fps["FrameRate"] = 60.0

        if not project.SetRenderSettings(try_with_fps):
            warn("render setting FrameRate=60 rejected by this build; retrying with minimal settings")
            if not project.SetRenderSettings(render_settings):
                fail("SetRenderSettings failed")

        job_id = project.AddRenderJob()
        if not job_id:
            fail("failed to add render job")

        if not project.StartRendering(job_id):
            fail("failed to start render")

        wait_for_render(project)

        status = project.GetRenderJobStatus(job_id) or {}
        print(status)

        # Resolve usually appends the container extension itself.
        output_candidate = output_dir / f"{output_name}.mp4"
        if output_candidate.exists():
            print(f"done: {output_candidate}")
        else:
            print(f"render finished; expected output base: {output_dir / output_name}")

    finally:
        try:
            project_manager.DeleteProject(project_name)
        except Exception:
            pass


if __name__ == "__main__":
    main()
