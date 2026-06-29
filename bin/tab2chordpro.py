#!/usr/bin/env python3
"""
tab2chordpro: Convert guitar tab chord-over-lyric format to ChordPro v6.

Usage:
    tab2chordpro.py < input.txt > output.chordpro
    tab2chordpro.py input.txt output.chordpro
"""

import sys
import re
from typing import List, Tuple, Optional

class Tab2ChordPro:
    def __init__(self):
        self.lines = []
        self.metadata = {}
        self.output = []

    def read_input(self, text: str) -> None:
        """Parse input text into lines."""
        self.lines = text.split('\n')

    def extract_metadata(self) -> None:
        """Extract Key, Capo, Tempo, Title, Artist from header lines."""
        for i, line in enumerate(self.lines[:30]):  # Only check first 30 lines
            # Title/Artist: "Song Title by Artist" or "Song Title" on one line, "by Artist" on next
            if ' by ' in line and '[' not in line:
                # Format: "Song Title by Artist"
                parts = line.split(' by ')
                title = parts[0].strip().replace('Official ', '').replace('Chords', '').strip()
                artist = parts[1].strip() if len(parts) > 1 else None
                if title and len(title) > 2:
                    self.metadata['title'] = title
                if artist:
                    self.metadata['artist'] = artist
            elif line.strip().startswith('by ') and '[' not in line:
                # Format: previous line is title, this line is "by Artist"
                if i > 0:
                    prev = self.lines[i-1].strip().replace('Official ', '').replace('Chords', '').strip()
                    if prev and len(prev) > 2 and '[' not in prev:
                        self.metadata['title'] = prev
                artist = line.strip()[3:].strip()  # Remove "by "
                if artist:
                    self.metadata['artist'] = artist

            if 'Key:' in line:
                match = re.search(r'Key:\s*([A-G][#b]?m?)', line)
                if match:
                    self.metadata['key'] = match.group(1)

            if 'Capo:' in line or 'capo' in line.lower():
                if 'no capo' in line.lower():
                    pass
                else:
                    match = re.search(r'(\d+)(?:st|nd|rd|th)?', line)
                    if match:
                        self.metadata['capo'] = match.group(1)

            if 'Tempo:' in line:
                match = re.search(r'Tempo:\s*(\d+)', line)
                if match:
                    self.metadata['tempo'] = match.group(1)

    def is_chord_line(self, line: str) -> bool:
        """Detect if line is mostly chords (sparse non-chord content)."""
        if not line.strip():
            return False
        stripped = line.lstrip()
        if not stripped or len(stripped) < 3:
            return False
        if stripped.startswith('[') and stripped.endswith(']'):
            return False
        if any(x in stripped for x in [' by ', 'Tuning:', 'Key:', 'views', 'saves']):
            return False

        # Strict chord pattern: chord must have letter + optional qual/ext
        chord_pattern = r'[A-G][#b]?(?:m|mi|min|aug|\+|dim|0|h)?(?:maj|7|9|sus|add)?(?:\d+)?(?:[b#]\d)?(?:/[A-G][#b]?)?'
        chords = re.findall(chord_pattern, stripped)

        # Heuristic: if 60%+ of non-space chars are chord chars, it's a chord line
        if not chords:
            return False

        chord_text = ''.join(chords)
        non_space = len(re.sub(r'\s', '', stripped))

        # Must have at least 2 chords or be all/mostly chords
        return len(chords) >= 2 or (non_space > 0 and len(chord_text) / non_space > 0.6)

    def extract_chords_with_positions(self, chord_line: str) -> List[Tuple[str, int]]:
        """Extract chord names and their column positions."""
        chords = []
        # Comprehensive chord pattern
        pattern = r'[A-G][#b]?(?:m|mi|min|aug|\+|dim|0|h)?(?:maj)?(?:\d+)?(?:sus\d)?(?:add\d)?(?:b\d|#\d)?(?:/[A-G][#b]?)?'

        for match in re.finditer(pattern, chord_line):
            chord = match.group(0)
            pos = match.start()
            chords.append((chord, pos))

        return chords

    def merge_chord_lyric(self, chord_line: str, lyric_line: str) -> str:
        """Merge chord line with lyric line using column positions."""
        chords = self.extract_chords_with_positions(chord_line)
        if not chords:
            return lyric_line.strip()

        # Strip leading spaces from lyric for alignment
        lyric_stripped = lyric_line.lstrip()
        lyric_lead_spaces = len(lyric_line) - len(lyric_stripped)

        # Build result by inserting chords from right to left (preserve indices)
        result = lyric_stripped

        for chord, pos in reversed(chords):
            # Adjust position for stripped leading spaces
            adj_pos = max(0, pos - lyric_lead_spaces)

            # Clamp to lyric length
            if adj_pos >= len(result):
                # Chord is past end of lyric
                result += f' [{chord}]'
            else:
                # Try to snap to word boundary (prefer start of word)
                snap_pos = self._snap_to_word_boundary(result, adj_pos)
                result = result[:snap_pos] + f'[{chord}]' + result[snap_pos:]

        return result

    def _snap_to_word_boundary(self, text: str, pos: int) -> int:
        """Snap position to nearest word boundary (prefer word start)."""
        if pos >= len(text):
            return pos

        # Check if we're already at a word boundary
        if pos == 0 or text[pos - 1].isspace():
            return pos

        # Look back for word start (max 3 chars)
        for i in range(max(0, pos - 3), pos):
            if text[i].isspace() or i == 0:
                return i + (0 if i == 0 and not text[i].isspace() else 1)

        # If no boundary found, use original position
        return pos

    def process(self) -> None:
        """Process lines and build ChordPro output."""
        # Find first section header to skip everything before it
        first_section = -1
        for i, line in enumerate(self.lines):
            if line.strip().startswith('[') and line.strip().endswith(']') and not self.is_chord_line(line.strip()):
                first_section = i
                break

        start_idx = max(0, first_section) if first_section >= 0 else 0
        i = start_idx

        while i < len(self.lines):
            line = self.lines[i]
            stripped = line.strip()

            # Skip remaining header-like lines
            if first_section < 0 or i < first_section:
                if any(x in line for x in ['views', 'saves', 'Tuning:', 'Tempo:', 'Transpose', 'Official', 'Key:', 'Capo:', 'by ']):
                    i += 1
                    continue
                if not stripped or len(stripped) < 3:
                    i += 1
                    continue

            # Empty line
            if not stripped:
                if self.output and self.output[-1] != '':
                    self.output.append('')
                i += 1
                continue

            # Section header: [Verse 1], [Chorus], etc
            if stripped.startswith('[') and stripped.endswith(']') and not self.is_chord_line(stripped):
                section = stripped[1:-1]
                section_type = self._parse_section_type(section)

                if section_type in ['verse', 'chorus', 'bridge', 'intro', 'outro']:
                    self.output.append(f'{{start_of_{section_type}: label="{section}"}}')
                else:
                    self.output.append(f'{{comment: {section}}}')
                i += 1
                continue

            # Chord line followed by lyric line
            if self.is_chord_line(stripped) and i + 1 < len(self.lines):
                next_line = self.lines[i + 1]
                next_stripped = next_line.strip()

                if next_stripped and not self.is_chord_line(next_stripped):
                    # This is a chord-over-lyric pair
                    merged = self.merge_chord_lyric(stripped, next_stripped)
                    self.output.append(merged)
                    i += 2
                    continue

            # Chord-only line (no following lyric)
            if self.is_chord_line(stripped):
                chords = self.extract_chords_with_positions(stripped)
                if chords:
                    chord_str = ' '.join([f'[{c[0]}]' for c in chords])
                    self.output.append(chord_str)
                i += 1
                continue

            # Regular lyric line
            if stripped:
                self.output.append(stripped)
            i += 1

        self._finalize_sections()

    def _parse_section_type(self, section: str) -> str:
        """Map section name to ChordPro type."""
        section_lower = section.lower()
        if 'verse' in section_lower:
            return 'verse'
        elif 'chorus' in section_lower:
            return 'chorus'
        elif 'bridge' in section_lower:
            return 'bridge'
        elif 'intro' in section_lower:
            return 'intro'
        elif 'outro' in section_lower:
            return 'outro'
        elif 'interlude' in section_lower:
            return 'bridge'  # Map interlude to bridge for now
        else:
            return 'comment'

    def _finalize_sections(self) -> None:
        """Add section ends and track open sections properly."""
        open_sections = []
        finalized = []

        for line in self.output:
            if line.startswith('{start_of_'):
                # Close previous section if needed
                if open_sections and not any(x in open_sections[-1] for x in ['intro', 'outro']):
                    section = open_sections.pop()
                    finalized.append(f'{{end_of_{section}}}')

                match = re.search(r'\{start_of_(\w+)', line)
                if match:
                    section_type = match.group(1)
                    open_sections.append(section_type)
                finalized.append(line)

            elif line.startswith('{end_of_'):
                # Already has end tag
                if open_sections:
                    open_sections.pop()
                finalized.append(line)

            elif line.startswith('{comment:'):
                finalized.append(line)

            elif not line.strip():
                # Empty line - don't close section yet
                finalized.append(line)

            else:
                # Regular content line
                finalized.append(line)

        # Close any remaining open sections
        while open_sections:
            section = open_sections.pop()
            finalized.append(f'{{end_of_{section}}}')

        self.output = finalized

    def add_metadata_header(self) -> None:
        """Prepend metadata to output."""
        header = []

        if 'title' in self.metadata:
            header.append(f'{{title: {self.metadata["title"]}}}')
        if 'artist' in self.metadata:
            header.append(f'{{artist: {self.metadata["artist"]}}}')
        if 'key' in self.metadata:
            header.append(f'{{key: {self.metadata["key"]}}}')
        if 'capo' in self.metadata:
            header.append(f'{{capo: {self.metadata["capo"]}}}')
        if 'tempo' in self.metadata:
            header.append(f'{{tempo: {self.metadata["tempo"]}}}')

        if header:
            header.append('')
            self.output = header + self.output

    def get_output(self) -> str:
        """Return formatted ChordPro output."""
        return '\n'.join(self.output).strip() + '\n'


def main():
    if len(sys.argv) > 1:
        # Read from file
        input_file = sys.argv[1]
        with open(input_file, 'r') as f:
            text = f.read()
    else:
        # Read from stdin
        text = sys.stdin.read()

    converter = Tab2ChordPro()
    converter.read_input(text)
    converter.extract_metadata()
    converter.process()
    converter.add_metadata_header()

    if len(sys.argv) > 2:
        # Write to output file
        output_file = sys.argv[2]
        with open(output_file, 'w') as f:
            f.write(converter.get_output())
        print(f'Written to {output_file}', file=sys.stderr)
    else:
        # Write to stdout
        print(converter.get_output(), end='')


if __name__ == '__main__':
    main()
