# VirtualDJ Filter Syntax Reference

Quick reference for VirtualDJ filter folder syntax and examples.

## Popular Filters

| Filter | Result |
|--------|--------|
| `isscanned=0` | All unanalyzed files |
| `isscanned=1` | All analyzed files |
| `top 100 nbplay` | Top 100 most played (by play count) |
| `top 100 firstseen` | Top 100 recently added |
| `days since lastplay<7` | Played in last week |
| `days since lastplay<31` | Played in last month |
| `days since lastplay<365` | Played in last year |
| `lastplay=0` | Never played |
| `Play Count>=1` | Played at least once |
| `type=audio` | Audio files only |
| `type=video` | Video files only |
| `type=karaoke` | Karaoke files only |
| `year>=1980 and year<1990` | Files from 1980s (1980-1989) |
| `hascover=1` | Files with cover art |
| `hascover=0` | Files without cover art |
| `Precomputed Stems=1` | Files with pre-computed stems |
| `bpm>120 and bpm<130` | BPM range 120-130 |

## Database Filters

| Filter | Result |
|--------|--------|
| `isscanned=0` | Unanalyzed files |
| `isscanned=1` | Analyzed files |
| `exists=1` | Files in search database |
| `exists=0` | Missing files from database |
| `days since first seen <10` | Added in last 10 days |
| `group by year range 10` | Year tags grouped by 10 (e.g., 2000-2010) |

## Tag Management Filters

| Filter | Result |
|--------|--------|
| `Artist is ""` | No artist tag |
| `Title is ""` | No title tag |
| `Genre is ""` | No genre tag |
| `bitrate < 128 and bitrate >1` | Low bitrate (< 128 kbps) |
| `Album Art = 0` | No cover art in tag |

## Mixing Filters

| Filter | Result |
|--------|--------|
| `group by bpm` | Nested folders by BPM |
| `bpm>=90 and bpm <=95` | BPM range 90-95 |
| `Bpm Difference <2 and Key Difference <1` | Compatible with active deck (±2 BPM, ±1 key) |

## Color Filters

| Filter | Result |
|--------|--------|
| `color = "none"` | No color assigned |
| `color = "red"` | Red color (by name) |
| `color = "#FF0000"` | Red color (by hex value) |
| `color = "255,0,0"` | Red color (by RGB value) |
| `group by color` | Nested folders by color |

## Syntax Patterns

### Comparison Operators
- `=` equals
- `<` less than
- `>` greater than
- `<=` less than or equal
- `>=` greater than or equal

### Logical Operators
- `and` - combine conditions
- Example: `bpm>120 and bpm<130`

### Date/Time Fields
- `days since lastplay` - days since last played
- `days since first seen` - days since added to database

### Special Functions
- `top N <field>` - top N results by field
- `group by <field>` - organize into nested folders
- `is ""` - empty/missing field check
