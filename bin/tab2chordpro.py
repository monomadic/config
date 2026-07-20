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
        self._open = None  # currently open ChordPro environment, if any

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

    # A single, fully-anchored chord token (e.g. F, Bb, Dm7, Fsus2, G/B).
    _CHORD_TOKEN = re.compile(
        r'^[A-G][#b]?(?:m|mi|min|aug|\+|dim|0|h)?(?:maj)?(?:\d+)?'
        r'(?:sus\d?)?(?:add\d)?(?:[b#]\d)?(?:/[A-G][#b]?)?$'
    )

    def is_chord_line(self, line: str) -> bool:
        """A chord line is one whose every token is a complete chord.

        This treats a lone chord (``F``, ``Bb``) the same as a row of them,
        and never mistakes a lyric that merely contains chord letters for a
        chord line.
        """
        stripped = line.strip()
        if not stripped:
            return False
        if stripped.startswith('[') and stripped.endswith(']'):
            return False
        if any(x in stripped for x in [' by ', 'Tuning:', 'Key:', 'views', 'saves']):
            return False

        tokens = stripped.split()
        return bool(tokens) and all(self._CHORD_TOKEN.match(t) for t in tokens)

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
        """Merge chord line with lyric line by monospaced column position.

        Chord and lyric lines are assumed to share a monospaced coordinate
        system: a chord at column N belongs over whatever the lyric has at
        column N. Chords whose column falls within a word are anchored to the
        start of that word; chords landing on whitespace attach to the next
        word; chords past the end of the lyric are appended at the end.
        """
        chords = self.extract_chords_with_positions(chord_line)
        if not chords:
            return lyric_line.rstrip()

        # Absolute columns matter, so don't strip leading whitespace.
        result = lyric_line.rstrip()
        end = len(result)

        inserts = []   # (anchor_column, source_column, chord)
        trailing = []  # chords past the end of the lyric

        for chord, pos in chords:
            if pos >= end:
                trailing.append(chord)
            else:
                inserts.append((self._word_anchor(result, pos), pos, chord))

        # Insert right-to-left so earlier indices stay valid. When several
        # chords snap to the same anchor (e.g. a leading-whitespace pickup
        # chord and the chord over the first word), break the tie by source
        # column so the right-most chord is inserted first and left-to-right
        # order survives: [C][C/E]When, not [C/E][C]When.
        for anchor, _src, chord in sorted(inserts, key=lambda x: (x[0], x[1]), reverse=True):
            result = result[:anchor] + f'[{chord}]' + result[anchor:]

        if trailing:
            result = result.rstrip() + ' ' + ''.join(f'[{c}]' for c in trailing)

        return result.lstrip()

    def _word_anchor(self, text: str, pos: int) -> int:
        """Return the start column of the word at/after ``pos``."""
        n = len(text)
        if pos >= n:
            return n
        # Landed on whitespace: move forward to the next word.
        while pos < n and text[pos].isspace():
            pos += 1
        if pos >= n:
            return n
        # Landed inside a word: move back to its start.
        while pos > 0 and not text[pos - 1].isspace():
            pos -= 1
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
                self._open_section(section)
                i += 1
                continue

            # Chord line followed by lyric line
            if self.is_chord_line(stripped) and i + 1 < len(self.lines):
                next_line = self.lines[i + 1]
                next_stripped = next_line.strip()

                if next_stripped and not self.is_chord_line(next_stripped):
                    # Chord-over-lyric pair. Pass the RAW lines so the
                    # monospaced column alignment is preserved.
                    merged = self.merge_chord_lyric(line, next_line)
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

        # Close whatever section is still open at the end of the song.
        self._close_section()
        # Drop any trailing blank left behind.
        while self.output and self.output[-1] == '':
            self.output.pop()

    # ChordPro environments we wrap in start_of_/end_of_ tags. Everything
    # else (intro, outro, interlude, tab, solo, ...) is left sectionless —
    # the ChordPro docs recommend not fabricating environments for these.
    ENVIRONMENTS = {'verse', 'chorus', 'bridge'}

    def _parse_section_type(self, section: str) -> str:
        """Map a section label to a ChordPro environment name, or ''."""
        section_lower = section.lower()
        if 'verse' in section_lower:
            return 'verse'
        elif 'chorus' in section_lower:
            return 'chorus'
        elif 'bridge' in section_lower:
            return 'bridge'
        else:
            return ''

    def _open_section(self, section: str) -> None:
        """Close any open section and open the new one.

        Environment sections (verse/chorus/bridge) are wrapped; chorus needs
        no label. Anything else becomes a plain comment and stays sectionless.
        """
        self._close_section()
        section_type = self._parse_section_type(section)

        if section_type == 'chorus':
            self.output.append('{start_of_chorus}')
            self._open = 'chorus'
        elif section_type in self.ENVIRONMENTS:
            self.output.append(f'{{start_of_{section_type}: label="{section}"}}')
            self._open = section_type
        else:
            self.output.append(f'{{comment: {section}}}')

    def _close_section(self) -> None:
        """Emit an end tag for the currently open environment, if any."""
        if not getattr(self, '_open', None):
            return
        # Don't let trailing blank lines land inside the environment.
        while self.output and self.output[-1] == '':
            self.output.pop()
        self.output.append(f'{{end_of_{self._open}}}')
        self.output.append('')
        self._open = None

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
