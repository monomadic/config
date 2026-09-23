#!/usr/bin/env python3
"""
interpolate-resolve.py

Import one input clip into DaVinci Resolve, apply Optical Flow interpolation,
and render to HEVC MP4 or ProRes 422 Proxy MOV. Attempts to make the
project/timeline target fps as early as possible, but tolerates Resolve builds
that lock frame rate unexpectedly. Defaults to 60 fps.

Usage:
  interpolate-resolve.py /absolute/input.mp4 /absolute/output.mp4
  interpolate-resolve.py --fps 120 /absolute/input.mp4 /absolute/output.mp4
  interpolate-resolve.py --quality speed-warp-better /absolute/input.mp4 /absolute/output.mp4
  interpolate-resolve.py --quality speed-warp-metal --range larger /absolute/input.mp4 /absolute/output.mp4
  interpolate-resolve.py --prores /absolute/input.mp4 /absolute/output.mov
  interpolate-resolve.py --debug-formats /absolute/input.mp4 /absolute/output.mp4
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
import uuid
from pathlib import Path


# Values and order from Resolve's scripting README ("RetimeProcess" and
# "MotionEstimation" clip properties). The enums are declared there as bare
# names after an explicit `= 0`, so each subsequent name is the next integer.
RETIME_OPTICAL_FLOW = 3

MOTION_EST_USE_PROJECT = 0

# (clip enum, project setting value, label). Speed Warp Metal exists only as
# a project setting (`imageMotionEstimationMode`) — the clip-level enum in the
# README stops at Speed Warp Faster — so its clip value is USE_PROJECT and the
# project setting carries the mode. The project setting is written for every
# mode anyway, so a clip left at USE_PROJECT still renders what was asked.
MOTION_ESTIMATION: dict[str, tuple[int, str, str]] = {
    "standard-faster": (1, "standardFaster", "Standard Faster"),
    "standard-better": (2, "standardBetter", "Standard Better"),
    "enhanced-faster": (3, "enhancedFaster", "Enhanced Faster"),
    "enhanced-better": (4, "enhancedBetter", "Enhanced Better"),
    "speed-warp-better": (5, "speedWarpBetter", "Speed Warp Better"),
    "speed-warp-faster": (6, "speedWarpFaster", "Speed Warp Faster"),
    "speed-warp-metal": (MOTION_EST_USE_PROJECT, "speedWarpMetal", "Speed Warp Metal"),
    # Kept so older invocations and docs keep working.
    "speed-warp": (5, "speedWarpBetter", "Speed Warp Better"),
}

# Motion range is a project-only setting (`imageMotionEstimationRange`); there
# is no clip property for it. Resolve's value for the UI's "Large" is `larger`
# (probed on 21.0.2: small/medium/larger accepted, large/wide rejected).
MOTION_RANGE: dict[str, tuple[str, str]] = {
    "small": ("small", "Small"),
    "medium": ("medium", "Medium"),
    "larger": ("larger", "Large"),
    "large": ("larger", "Large"),
    "wide": ("larger", "Large"),
}


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


def fmt_hms(seconds: float) -> str:
    seconds = max(0, int(seconds))
    return f"{seconds // 3600:02d}:{seconds % 3600 // 60:02d}:{seconds % 60:02d}"


def wait_for_render(project, job_id: str, total_frames: int | None, poll_s: float = 1.0) -> None:
    """
    Poll the render job and show a live status line on stderr. Resolve reports
    CompletionPercentage, TimeTakenToRenderInMs and EstimatedTimeRemainingInMs
    per job; frames and fps are derived from the percentage against the
    expected output frame count, so they are estimates, not a frame counter.
    """
    live = sys.stderr.isatty()
    started = time.monotonic()
    while project.IsRenderingInProgress():
        status = project.GetRenderJobStatus(job_id) or {}
        pct = float(status.get("CompletionPercentage", 0) or 0)
        taken_ms = status.get("TimeTakenToRenderInMs")
        elapsed = (taken_ms / 1000.0) if taken_ms else (time.monotonic() - started)
        remaining_ms = status.get("EstimatedTimeRemainingInMs")
        eta = fmt_hms(remaining_ms / 1000.0) if remaining_ms is not None else "--:--:--"
        line = f"{status.get('JobStatus', 'Rendering')}: {pct:5.1f}%  elapsed {fmt_hms(elapsed)}  eta {eta}"
        if total_frames:
            done = int(total_frames * pct / 100.0)
            fps = done / elapsed if elapsed > 0 else 0.0
            line += f"  ~{done}/{total_frames} frames  ~{fps:.1f} fps"
        if live:
            print("\r\033[K" + line, end="", file=sys.stderr, flush=True)
        time.sleep(poll_s)
    if live:
        print("\r\033[K", end="", file=sys.stderr, flush=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render one input clip through DaVinci Resolve as HEVC MP4 or ProRes 422 Proxy MOV with Optical Flow interpolation."
    )
    parser.add_argument(
        "--fps",
        default="60",
        help="Target output frame rate. Defaults to 60.",
    )
    quality = parser.add_mutually_exclusive_group()
    quality.add_argument(
        "--quality",
        choices=tuple(MOTION_ESTIMATION.keys()),
        default="enhanced-better",
        help="Optical Flow motion estimation quality. Defaults to enhanced-better.",
    )
    quality.add_argument(
        "--speed-warp",
        action="store_true",
        help="Shortcut for --quality speed-warp-better",
    )
    parser.add_argument(
        "--range",
        dest="motion_range",
        choices=tuple(MOTION_RANGE.keys()),
        default="medium",
        help="Optical Flow motion range (small, medium, larger; wide/large = larger). Defaults to medium.",
    )
    parser.add_argument(
        "--orientation",
        choices=("auto", "portrait", "landscape"),
        default="auto",
        help=(
            "Output canvas orientation. 'auto' (default) matches the input clip's "
            "resolution and orientation. 'portrait'/'landscape' force the canvas, "
            "swapping the detected dimensions if needed."
        ),
    )
    parser.add_argument(
        "--debug-formats",
        action="store_true",
        help="Print available render formats/codecs before rendering",
    )
    parser.add_argument(
        "--prores",
        action="store_true",
        help="Render QuickTime/MOV using ProRes 422 Proxy, the lowest bitrate ProRes profile",
    )
    parser.add_argument("input_file", help="Absolute input file path")
    parser.add_argument("output_file", help="Absolute output file path")
    args = parser.parse_args()
    args.fps = normalize_fps(args.fps)
    if args.speed_warp:
        args.quality = "speed-warp-better"
    return args


def normalize_fps(value: str) -> str:
    fps = value.lower().removesuffix("fps")
    try:
        parsed = float(fps)
    except ValueError:
        fail(f"invalid fps: {value}")
    if parsed <= 0:
        fail(f"fps must be greater than zero: {value}")
    if parsed.is_integer():
        return str(int(parsed))
    return str(parsed)


def probe_duration(input_file: Path) -> float | None:
    """Return the input's duration in seconds via ffprobe, or None."""
    ffprobe = shutil.which("ffprobe")
    if ffprobe is None:
        return None
    try:
        out = subprocess.run(
            [ffprobe, "-v", "error", "-show_entries", "format=duration",
             "-of", "default=noprint_wrappers=1:nokey=1", str(input_file)],
            capture_output=True, text=True, check=True,
        ).stdout.strip()
        return float(out) if out else None
    except (subprocess.CalledProcessError, ValueError):
        return None


def probe_dimensions(input_file: Path) -> tuple[int, int] | None:
    """
    Return the input clip's effective (width, height) using ffprobe, accounting
    for rotation metadata (phones store portrait video as a rotated landscape
    frame). Returns None if ffprobe is unavailable or the probe fails, in which
    case the caller falls back to Resolve's default canvas.
    """
    ffprobe = shutil.which("ffprobe")
    if ffprobe is None:
        return None

    cmd = [
        ffprobe,
        "-v", "error",
        "-select_streams", "v:0",
        "-show_streams",
        "-of", "json",
        str(input_file),
    ]
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
        data = json.loads(proc.stdout)
    except (subprocess.CalledProcessError, json.JSONDecodeError, OSError):
        return None

    streams = data.get("streams") or []
    if not streams:
        return None
    stream = streams[0]

    try:
        width = int(stream["width"])
        height = int(stream["height"])
    except (KeyError, ValueError, TypeError):
        return None

    rotation = 0
    for side_data in stream.get("side_data_list") or []:
        if "rotation" in side_data:
            try:
                rotation = int(side_data["rotation"])
            except (ValueError, TypeError):
                rotation = 0
            break
    else:
        # Older containers expose rotation as a stream tag instead of side data.
        tag = (stream.get("tags") or {}).get("rotate")
        try:
            rotation = int(tag) if tag is not None else 0
        except (ValueError, TypeError):
            rotation = 0

    if abs(rotation) % 180 == 90:
        width, height = height, width

    if width <= 0 or height <= 0:
        return None
    return width, height


def orient_dimensions(width: int, height: int, orientation: str) -> tuple[int, int]:
    """
    Force (width, height) to the requested orientation, swapping if the detected
    dimensions disagree. 'auto' leaves them untouched.
    """
    if orientation == "portrait" and width > height:
        return height, width
    if orientation == "landscape" and height > width:
        return height, width
    return width, height


def try_set_project_resolution(project, width: int, height: int) -> bool:
    """
    Set the timeline/canvas resolution before timeline creation so the new
    timeline inherits it. As with FPS, some builds may reject this.
    """
    ok = False
    try:
        ok = bool(project.SetSetting("timelineResolutionWidth", str(width)))
        ok = bool(project.SetSetting("timelineResolutionHeight", str(height))) and ok
    except Exception:
        ok = False

    # Keep delivery resolution matched to the timeline; ignore failures.
    for key, value in (
        ("timelineOutputResolutionWidth", str(width)),
        ("timelineOutputResolutionHeight", str(height)),
    ):
        try:
            project.SetSetting(key, value)
        except Exception:
            pass

    return ok


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


def find_quicktime_format_token(project) -> str:
    formats = project.GetRenderFormats() or {}
    for desc, token in formats.items():
        t = str(token).lower()
        d = str(desc).lower()
        if t in {"mov", "quicktime"} or "quicktime" in d:
            return str(token)
    fail(f"could not find QuickTime/MOV render format; available formats: {formats}")


def find_prores_proxy_codec_token(project, format_token: str) -> str:
    codecs = project.GetRenderCodecs(format_token) or {}
    for desc, token in codecs.items():
        t = str(token).lower()
        d = str(desc).lower()
        if "prores" in t and "proxy" in t:
            return str(token)
        if "prores" in d and "proxy" in d:
            return str(token)
    fail(f"could not find ProRes 422 Proxy codec in {format_token} codecs: {codecs}")


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


def try_set_project_interpolation(project, mode_value: str, range_value: str) -> bool:
    """
    Project-level frame interpolation (Master Settings > Frame Interpolation).
    Motion range only exists here, and so does Speed Warp Metal; clips set to
    USE_PROJECT inherit both. Attempted before timeline creation.
    """
    ok = True
    for key, value in (
        ("imageRetimeInterpolation", "opticalFlow"),
        ("imageMotionEstimationMode", mode_value),
        ("imageMotionEstimationRange", range_value),
    ):
        try:
            ok = bool(project.SetSetting(key, value)) and ok
        except Exception:
            ok = False
    return ok


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


def unique_output_name(output_dir: Path, base_name: str, extension: str) -> str:
    """
    Return a name (without extension) that does not collide with an existing
    file in output_dir. If base_name+extension is free, return base_name as-is;
    otherwise append -1, -2, ... until a free name is found.
    """
    candidate = base_name
    counter = 1
    while (output_dir / f"{candidate}{extension}").exists():
        candidate = f"{base_name}-{counter}"
        counter += 1
    return candidate


def ensure_empty_dir_exists(dir_path: Path) -> None:
    if not dir_path.exists():
        dir_path.mkdir(parents=True, exist_ok=True)
    if not dir_path.is_dir():
        fail(f"output directory is not a directory: {dir_path}")


def print_settings(
    *,
    input_file: Path,
    output_file: Path,
    fps: str,
    fps_forced: bool,
    canvas: str | None,
    orientation: str,
    retime: str,
    retime_value: int,
    motion_estimation: str,
    motion_estimation_value: int,
    motion_range: str,
    motion_range_value: str,
    format_token: str,
    codec_token: str,
) -> None:
    """Report the settings this render is actually using, before it starts."""
    canvas_line = canvas or "Resolve default (input resolution not detected)"
    if orientation != "auto":
        canvas_line += f" (forced {orientation})"
    rows = [
        ("input", str(input_file)),
        ("output", str(output_file)),
        ("frame rate", f"{fps} fps" + ("" if fps_forced else " (requested; Resolve rejected the override)")),
        ("resolution", canvas_line),
        ("retime process", f"{retime} (RetimeProcess={retime_value})"),
        ("motion estimation", f"{motion_estimation} (MotionEstimation={motion_estimation_value})"),
        ("motion range", f"{motion_range} (imageMotionEstimationRange={motion_range_value})"),
        ("format / codec", f"{format_token} / {codec_token}"),
    ]
    width = max(len(label) for label, _ in rows)
    print("render settings:")
    for label, value in rows:
        print(f"  {label.rjust(width)}: {value}")


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
        project_fps_set = try_set_project_fps(project, args.fps)
        if not project_fps_set:
            warn(f"could not set project timelineFrameRate={args.fps} before import; continuing")

        motion_estimation, motion_estimation_setting, motion_estimation_label = MOTION_ESTIMATION[args.quality]
        motion_range_setting, motion_range_label = MOTION_RANGE[args.motion_range]
        if not try_set_project_interpolation(project, motion_estimation_setting, motion_range_setting):
            warn(
                f"could not set project frame interpolation to {motion_estimation_setting}/{motion_range_setting}; "
                "the clip-level mode still applies, but the motion range may stay at the default"
            )

        # Detect input resolution/orientation and set the canvas before timeline
        # creation so a portrait clip renders portrait instead of pillarboxed.
        dims = probe_dimensions(input_file)
        canvas = None
        if dims is None:
            warn("could not detect input resolution (ffprobe missing or probe failed); using Resolve's default canvas")
        else:
            width, height = orient_dimensions(dims[0], dims[1], args.orientation)
            canvas = f"{width}x{height}"
            if not try_set_project_resolution(project, width, height):
                warn(f"could not set project resolution to {width}x{height} before import; render may use the default canvas")

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

        timeline_fps_set = try_set_timeline_fps(timeline, args.fps)
        if not timeline_fps_set and not project_fps_set:
            warn(f"could not explicitly force timeline FPS to {args.fps}; render may still succeed at {args.fps} if allowed by your build")

        items = timeline.GetItemListInTrack("video", 1)
        if not items:
            fail("no timeline items found on video track 1")

        clip = items[0]

        ok = True
        ok &= bool(clip.SetProperty("RetimeProcess", RETIME_OPTICAL_FLOW))
        ok &= bool(clip.SetProperty("MotionEstimation", motion_estimation))
        if not ok:
            fail("failed to set clip interpolation properties")

        project.DeleteAllRenderJobs()

        if args.prores:
            format_token = find_quicktime_format_token(project)
            codec_token = find_prores_proxy_codec_token(project, format_token)
            output_extension = ".mov"
        else:
            format_token = find_format_token(project, "mp4")
            codec_token = find_hevc_codec_token(project, format_token)
            output_extension = ".mp4"

        # Avoid clobbering an existing render; pick a free name if needed.
        resolved_name = unique_output_name(output_dir, output_name, output_extension)
        if resolved_name != output_name:
            warn(
                f"{output_name}{output_extension} exists; writing to "
                f"{resolved_name}{output_extension} instead"
            )
        output_name = resolved_name

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
        try_with_fps["FrameRate"] = float(args.fps)

        if not project.SetRenderSettings(try_with_fps):
            warn(f"render setting FrameRate={args.fps} rejected by this build; retrying with minimal settings")
            if not project.SetRenderSettings(render_settings):
                fail("SetRenderSettings failed")

        print_settings(
            input_file=input_file,
            output_file=output_dir / f"{output_name}{output_extension}",
            fps=args.fps,
            fps_forced=project_fps_set or timeline_fps_set,
            canvas=canvas,
            orientation=args.orientation,
            retime="Optical Flow",
            retime_value=RETIME_OPTICAL_FLOW,
            motion_estimation=motion_estimation_label,
            motion_estimation_value=motion_estimation,
            motion_range=motion_range_label,
            motion_range_value=motion_range_setting,
            format_token=format_token,
            codec_token=codec_token,
        )

        job_id = project.AddRenderJob()
        if not job_id:
            fail("failed to add render job")

        if not project.StartRendering(job_id):
            fail("failed to start render")

        duration_s = probe_duration(input_file)
        total_frames = int(round(duration_s * float(args.fps))) if duration_s else None
        wait_for_render(project, job_id, total_frames)

        status = project.GetRenderJobStatus(job_id) or {}
        taken_ms = status.get("TimeTakenToRenderInMs")
        summary = f"render {status.get('JobStatus', 'finished')}"
        if taken_ms:
            summary += f" in {fmt_hms(taken_ms / 1000.0)}"
            if total_frames:
                summary += f" ({total_frames} frames, {total_frames / (taken_ms / 1000.0):.1f} fps)"
        print(summary)
        if status.get("Error"):
            warn(f"resolve reported: {status['Error']}")

        # Resolve usually appends the container extension itself.
        output_candidate = output_dir / f"{output_name}{output_extension}"
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
