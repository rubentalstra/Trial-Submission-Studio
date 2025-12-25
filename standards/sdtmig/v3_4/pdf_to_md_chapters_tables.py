#!/usr/bin/env python3
"""
Convert a PDF into per-chapter Markdown files.

Key fixes vs. the earlier version:
- pdfplumber "text" table strategy is OFF by default (it often turns entire pages into fake tables).
- Tables are extracted conservatively (line-based), and we extract text *around* tables to avoid duplication.
- Page text is cleaned for Markdown (header/footer stripping, de-hyphenation, paragraph wrapping).
- Optional: enable the risky "text" strategy with --enable-text-strategy.
"""

from __future__ import annotations

import argparse
import gc
import logging
import re
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

import pdfplumber
from pypdf import PdfReader, PdfWriter

try:
    import camelot  # type: ignore
except Exception:  # pragma: no cover
    camelot = None  # type: ignore

logger = logging.getLogger("pdf_to_md")

# ----------------------------
# Config: header/footer cleanup
# ----------------------------
HEADER_PATTERNS = [
    re.compile(
        r"^CDISC Study Data Tabulation Model Implementation Guide:", re.IGNORECASE
    ),
]
FOOTER_PATTERNS = [
    re.compile(
        r"^©\s*\d{4}\s+Clinical Data Interchange Standards Consortium", re.IGNORECASE
    ),
    re.compile(r"^\d{4}-\d{2}-\d{2}\s*$"),
    re.compile(r"^\s*Page\s+\d+\s*$", re.IGNORECASE),
]

BULLET_TOKENS = {"•", "o", "-", "–", "—", "*", "·"}


# ----------------------------
# Data structures
# ----------------------------
@dataclass(frozen=True)
class ChapterRange:
    number: int
    title: str
    start_page_1based: int
    end_page_1based: int


@dataclass(frozen=True)
class TableCandidate:
    grid: List[List[str]]
    bbox: Optional[Tuple[float, float, float, float]]  # (x0, y0, x1, y1)
    source: str  # e.g. "pdfplumber_lines", "camelot_lattice"
    page_w: Optional[float] = None
    page_h: Optional[float] = None


# ----------------------------
# TOC parsing (chapter detection)
# ----------------------------
TOC_HEADING_RE = re.compile(r"^\s*CONTENTS\s*$", re.IGNORECASE | re.MULTILINE)
TOC_TOPLEVEL_RE = re.compile(r"^(?P<num>\d{1,2})\s+(?P<title>.+?)\s+(?P<page>\d{1,4})$")


def _normalize_toc_line(line: str) -> str:
    # Remove dotted leaders and normalize whitespace
    line = re.sub(r"[.\u2024\u2027\u00b7]{2,}", " ", line)
    line = re.sub(r"\s+", " ", line).strip()
    return line


def detect_chapters_from_toc(
    src_pdf: Path, expected: int = 10, scan_pages: int = 25
) -> List[Tuple[int, str, int]]:
    """
    Returns list of: (chapter_number, chapter_title, start_page_1based)
    """
    chapters: Dict[int, Tuple[str, int]] = {}

    with pdfplumber.open(str(src_pdf)) as pdf:
        max_i = min(scan_pages, len(pdf.pages))
        toc_start = None

        for i in range(max_i):
            text = pdf.pages[i].extract_text() or ""
            if TOC_HEADING_RE.search(text):
                toc_start = i
                logger.debug("Found CONTENTS heading on PDF page %d", i + 1)
                break

        if toc_start is None:
            return []

        for i in range(toc_start, min(toc_start + 14, len(pdf.pages))):
            text = pdf.pages[i].extract_text() or ""
            for raw_line in text.splitlines():
                line = _normalize_toc_line(raw_line)
                if not line:
                    continue
                m = TOC_TOPLEVEL_RE.match(line)
                if not m:
                    continue
                num = int(m.group("num"))
                title = m.group("title").strip()
                page = int(m.group("page"))
                title = re.sub(r"\s+", " ", title)
                if 1 <= num <= 50:
                    chapters[num] = (title, page)

    out = [(n, chapters[n][0], chapters[n][1]) for n in sorted(chapters)]
    if expected and len(out) > expected:
        out = out[:expected]
    return out


def build_chapter_ranges(
    starts: List[Tuple[int, str, int]], total_pages: int
) -> List[ChapterRange]:
    out: List[ChapterRange] = []
    for idx, (num, title, start_p) in enumerate(starts):
        end_p = starts[idx + 1][2] - 1 if idx + 1 < len(starts) else total_pages
        out.append(ChapterRange(num, title, start_p, end_p))
    return out


def slugify(s: str) -> str:
    s = s.lower().strip()
    s = re.sub(r"[^\w\s-]", "", s)
    s = re.sub(r"\s+", "-", s)
    s = re.sub(r"-{2,}", "-", s)
    return s[:80].strip("-") or "chapter"


def write_index(out_dir: Path, src_pdf: Path, chapters: List[ChapterRange]) -> None:
    idx_path = out_dir / "index.md"
    lines = [
        "# SDTMIG_v3.4 — Chapters",
        "",
        f"- Source PDF: `{src_pdf.name}`",
        "",
        "## Chapter files",
        "",
    ]
    for ch in chapters:
        fname = f"chapter_{ch.number:02d}_{slugify(ch.title)}.md"
        lines.append(
            f"- [Chapter {ch.number}: {ch.title}]({fname}) (pages {ch.start_page_1based}–{ch.end_page_1based})"
        )
    idx_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ----------------------------
# Text cleanup
# ----------------------------
HEADING_RE = re.compile(r"^\s*\d+(\.\d+)*\s+\S+")
LIST_RE = re.compile(r"^\s*([•\-\u2013\u2014*o]|\d+[\).\]])\s+")


def _strip_headers_footers(lines: List[str]) -> List[str]:
    cleaned: List[str] = []
    for ln in lines:
        s = ln.strip()
        if not s:
            cleaned.append("")
            continue
        if any(p.search(s) for p in HEADER_PATTERNS):
            continue
        if any(p.search(s) for p in FOOTER_PATTERNS):
            continue
        cleaned.append(s)
    return cleaned


def _dehyphenate(lines: List[str]) -> List[str]:
    """
    Join hyphen-wrapped words split at end-of-line:
      "inter-" + "change" => "interchange"
    Only joins if next line starts with a lowercase letter.
    """
    out: List[str] = []
    i = 0
    while i < len(lines):
        ln = lines[i]
        if ln.endswith("-") and i + 1 < len(lines):
            nxt = lines[i + 1]
            if nxt and re.match(r"^[a-z]", nxt):
                out.append(ln[:-1] + nxt)
                i += 2
                continue
        out.append(ln)
        i += 1
    return out


def _wrap_paragraphs(lines: List[str]) -> str:
    """
    Turn PDF line-broken text into readable Markdown-ish paragraphs.
    Keeps headings and list items as hard breaks.
    """
    out_lines: List[str] = []
    buf: List[str] = []

    def flush():
        nonlocal buf
        if buf:
            out_lines.append(" ".join(buf).strip())
            buf = []

    for ln in lines:
        if not ln.strip():
            flush()
            out_lines.append("")
            continue

        if HEADING_RE.match(ln) or LIST_RE.match(ln):
            flush()
            out_lines.append(ln.strip())
            continue

        buf.append(ln.strip())

    flush()

    final: List[str] = []
    empty = 0
    for ln in out_lines:
        if ln.strip() == "":
            empty += 1
            if empty <= 2:
                final.append("")
        else:
            empty = 0
            final.append(ln)
    return "\n".join(final).strip()


def clean_page_text(raw_text: str) -> str:
    lines = [re.sub(r"\s+", " ", ln).strip() for ln in (raw_text or "").splitlines()]
    lines = _strip_headers_footers(lines)
    lines = _dehyphenate(lines)
    return _wrap_paragraphs(lines)


# ----------------------------
# Table helpers & filters
# ----------------------------
def sanitize_cell(val: object) -> str:
    s = "" if val is None else str(val)
    s = s.replace("\r", "\n")
    s = re.sub(r"\s+\n", "\n", s)
    s = re.sub(r"\n\s+", "\n", s)
    s = s.strip()
    s = s.replace("|", "\\|")
    s = s.replace("\n", "<br>")
    return s


def normalize_grid(raw: Sequence[Sequence[object]]) -> List[List[str]]:
    rows = [
        [sanitize_cell(c) for c in row]
        for row in raw
        if row and any(str(c or "").strip() for c in row)
    ]
    if not rows:
        return []
    ncols = max(len(r) for r in rows)
    return [r + [""] * (ncols - len(r)) for r in rows]


def grid_to_markdown(table: Sequence[Sequence[object]]) -> str:
    norm = normalize_grid(table)
    if not norm:
        return ""
    ncols = max(len(r) for r in norm)
    header = norm[0]
    body = norm[1:]

    def fmt_row(r: List[str]) -> str:
        return "| " + " | ".join(r) + " |"

    out = [fmt_row(header), "| " + " | ".join(["---"] * ncols) + " |"]
    out.extend(fmt_row(r) for r in body)
    return "\n".join(out)


def looks_like_bullet_list(grid: List[List[str]]) -> bool:
    """
    Reject grids that are basically a bullet list that got mis-identified as a table.
    """
    if not grid or len(grid) < 4:
        return False
    first_col = [r[0].strip() if r else "" for r in grid]
    bullets = sum(1 for v in first_col if (v[:1] in BULLET_TOKENS) or v.startswith("•"))
    ratio = bullets / max(1, len(first_col))
    second = [r[1].strip() for r in grid if len(r) > 1 and r[1].strip()]
    avg_second_len = sum(len(v) for v in second) / max(1, len(second))
    return ratio >= 0.6 and avg_second_len >= 40


def accept_table_candidate(cand: TableCandidate, *, table_debug: bool) -> bool:
    grid = cand.grid
    if not grid:
        return False

    nrows = len(grid)
    ncols = max(len(r) for r in grid) if grid else 0

    if nrows < 2 or ncols < 2:
        if table_debug:
            logger.debug("DROP[%s]: too small (%dx%d)", cand.source, nrows, ncols)
        return False

    if looks_like_bullet_list(grid):
        if table_debug:
            logger.debug("DROP[%s]: bullet-list detected", cand.source)
        return False

    if (
        cand.source in {"pdfplumber_text", "camelot_stream"}
        and cand.bbox
        and cand.page_w
        and cand.page_h
    ):
        x0, y0, x1, y1 = cand.bbox
        area_frac = ((x1 - x0) * (y1 - y0)) / (cand.page_w * cand.page_h)
        if area_frac > 0.55 and nrows >= 25 and ncols >= 6:
            if table_debug:
                logger.debug(
                    "DROP[%s]: looks like full-page layout (area=%.2f, %dx%d)",
                    cand.source,
                    area_frac,
                    nrows,
                    ncols,
                )
            return False

    nonempty = sum(1 for r in grid for c in r if c.strip())
    if nonempty < max(6, nrows * 2):
        if table_debug:
            logger.debug("DROP[%s]: too empty", cand.source)
        return False

    return True


# ----------------------------
# Page cropping and extraction
# ----------------------------
def crop_header_footer(page, *, top_crop: float, bottom_crop: float):
    """
    Crop away top/bottom fractions of the page to reduce header/footer noise.
    """
    top_crop = max(0.0, min(0.25, top_crop))
    bottom_crop = max(0.0, min(0.25, bottom_crop))
    w, h = page.width, page.height
    y0 = h * top_crop
    y1 = h * (1.0 - bottom_crop)
    return page.crop((0, y0, w, y1))


def extract_text_in_bbox(page, bbox: Tuple[float, float, float, float]) -> str:
    """Extract text from a bbox using non-layout mode for cleaner output."""
    # strict=False avoids occasional rounding issues when bboxes touch the crop boundary
    cropped = page.crop(bbox, strict=False)
    return cropped.extract_text(layout=False) or ""


def subtract_table_regions(
    page,
    *,
    table_bboxes: List[Tuple[float, float, float, float]],
    y_pad: float = 2.0,
) -> List[Tuple[float, float, float, float]]:
    """
    Create a list of bboxes that exclude table regions, so we can extract text
    without duplicating table content.

    Important: pdfplumber bboxes are in the *parent page coordinate system*.
    If `page` is already cropped, its `page.bbox` will not start at (0,0).
    So we always build regions in the page.bbox coordinate space.
    """
    x0b, y0b, x1b, y1b = page.bbox  # absolute coords
    if not table_bboxes:
        return [(x0b, y0b, x1b, y1b)]

    boxes = sorted(table_bboxes, key=lambda b: b[1])
    regions: List[Tuple[float, float, float, float]] = []
    cur_y = y0b

    for _x0, y0, _x1, y1 in boxes:
        y0p = max(y0b, y0 - y_pad)
        y1p = min(y1b, y1 + y_pad)
        if y0p > cur_y + 1:
            regions.append((x0b, cur_y, x1b, y0p))
        cur_y = max(cur_y, y1p)

    if cur_y < y1b - 1:
        regions.append((x0b, cur_y, x1b, y1b))
    return regions


# ----------------------------
# Table extraction methods
# ----------------------------
def extract_tables_pdfplumber(
    page,
    *,
    table_debug: bool,
    enable_text_strategy: bool,
) -> List[TableCandidate]:
    w, h = page.width, page.height
    out: List[TableCandidate] = []

    settings_lines = dict(
        vertical_strategy="lines",
        horizontal_strategy="lines",
        snap_tolerance=6,
        join_tolerance=3,
        edge_min_length=20,
        min_words_vertical=1,
        min_words_horizontal=1,
    )

    settings_text = dict(
        vertical_strategy="text",
        horizontal_strategy="text",
        snap_tolerance=6,
        join_tolerance=3,
        edge_min_length=10,
        min_words_vertical=2,
        min_words_horizontal=1,
        intersection_tolerance=5,
        text_tolerance=2,
    )

    settings_list = [("pdfplumber_lines", settings_lines)]
    if enable_text_strategy:
        settings_list.append(("pdfplumber_text", settings_text))

    for name, settings in settings_list:
        try:
            tables = page.find_tables(table_settings=settings)
        except Exception as e:  # pragma: no cover
            if table_debug:
                logger.debug("pdfplumber %s failed: %s", name, e)
            continue

        if table_debug:
            logger.debug("pdfplumber %s found %d candidate table(s)", name, len(tables))

        for tb in tables:
            raw = tb.extract()
            grid = normalize_grid(raw)
            if not grid:
                continue
            out.append(
                TableCandidate(
                    grid=grid,
                    bbox=tb.bbox,
                    source=name,
                    page_w=w,
                    page_h=h,
                )
            )

    return out


def extract_tables_camelot(
    single_page_pdf: Path, *, table_debug: bool
) -> List[TableCandidate]:
    if camelot is None:
        return []

    out: List[TableCandidate] = []

    def _read(flavor: str, **kwargs):
        try:
            return camelot.read_pdf(
                str(single_page_pdf),
                pages="1",
                flavor=flavor,
                suppress_stdout=True,
                **kwargs,
            )
        except Exception as e:
            if table_debug:
                logger.debug("Camelot %s failed: %s", flavor, e)
            return []

    for flavor, kwargs in [
        ("lattice", dict(line_scale=40)),
        ("stream", dict(edge_tol=500)),
    ]:
        tables = _read(flavor, **kwargs)
        for t in tables:
            try:
                df = t.df  # type: ignore[attr-defined]
                grid = normalize_grid(df.values.tolist())
                if grid:
                    out.append(
                        TableCandidate(grid=grid, bbox=None, source=f"camelot_{flavor}")
                    )
            except Exception:
                continue

    return out


def extract_aligned_text_table(page, *, table_debug: bool) -> List[TableCandidate]:
    text = page.extract_text(layout=False) or ""
    if ".xpt" not in text.lower():
        return []

    words = page.extract_words(use_text_flow=False, keep_blank_chars=False)
    if not words:
        return []

    y_tol = 3.0
    words_sorted = sorted(
        words, key=lambda w: (round(w["top"] / y_tol) * y_tol, w["x0"])
    )
    lines: List[List[dict]] = []
    for w in words_sorted:
        key = round(w["top"] / y_tol) * y_tol
        if not lines or abs(lines[-1][0]["top"] - key) > y_tol:
            lines.append([w])
        else:
            lines[-1].append(w)

    x_positions: List[float] = []
    for ln in lines:
        xs = sorted({round(w["x0"], 1) for w in ln})
        x_positions.extend(xs)
    if not x_positions:
        return []

    x_positions.sort()

    merged: List[float] = []
    for x in x_positions:
        if not merged or abs(merged[-1] - x) > 8:
            merged.append(x)

    grid: List[List[str]] = []
    for ln in lines:
        row = [""] * len(merged)
        for w in ln:
            idx = min(range(len(merged)), key=lambda i: abs(merged[i] - w["x0"]))
            row[idx] = (row[idx] + " " + w["text"]).strip() if row[idx] else w["text"]
        if any(c.strip() for c in row):
            grid.append([sanitize_cell(c) for c in row])

    if not grid:
        return []

    ncols = len(grid[0])
    keep_cols = []
    for c in range(ncols):
        nonempty = sum(1 for r in grid if r[c].strip())
        if nonempty >= max(2, len(grid) // 5):
            keep_cols.append(c)
    grid2 = [[r[c] for c in keep_cols] for r in grid] if keep_cols else grid

    if table_debug:
        logger.debug(
            "Aligned-text fallback produced grid %dx%d",
            len(grid2),
            len(grid2[0]) if grid2 else 0,
        )

    return [TableCandidate(grid=grid2, bbox=None, source="aligned")]


def extract_tables_for_page(
    page,
    single_page_pdf: Path,
    *,
    table_debug: bool,
    enable_text_strategy: bool,
) -> List[TableCandidate]:
    cands: List[TableCandidate] = []

    cands.extend(
        extract_tables_pdfplumber(
            page, table_debug=table_debug, enable_text_strategy=enable_text_strategy
        )
    )

    if not cands:
        cands.extend(extract_tables_camelot(single_page_pdf, table_debug=table_debug))

    if not cands:
        cands.extend(extract_aligned_text_table(page, table_debug=table_debug))

    kept: List[TableCandidate] = []
    for cand in cands:
        if accept_table_candidate(cand, table_debug=table_debug):
            kept.append(cand)

    seen = set()
    uniq: List[TableCandidate] = []
    for cand in kept:
        key = (
            re.sub(r"\s+", " ", " ".join(" ".join(r) for r in cand.grid))
            .strip()
            .lower()
        )
        if key and key not in seen:
            seen.add(key)
            uniq.append(cand)

    return uniq


# ----------------------------
# Main conversion loop
# ----------------------------
def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("pdf", type=Path, help="Input PDF file")
    ap.add_argument("out", type=Path, help="Output directory for markdown chapters")
    ap.add_argument(
        "--chapters",
        type=int,
        default=10,
        help="Expected chapters from TOC (default: 10)",
    )
    ap.add_argument(
        "--toc-scan-pages",
        type=int,
        default=25,
        help="Pages to scan for CONTENTS (default: 25)",
    )
    ap.add_argument(
        "--log-level",
        default="INFO",
        help="DEBUG, INFO, WARNING, ERROR (default: INFO)",
    )
    ap.add_argument("--log-file", default=None, help="Optional log file path")
    ap.add_argument(
        "--table-debug", action="store_true", help="Verbose table debug logging"
    )
    ap.add_argument(
        "--crop-top",
        type=float,
        default=0.06,
        help="Crop top fraction to remove headers (default: 0.06)",
    )
    ap.add_argument(
        "--crop-bottom",
        type=float,
        default=0.06,
        help="Crop bottom fraction to remove footers (default: 0.06)",
    )
    ap.add_argument(
        "--enable-text-strategy",
        action="store_true",
        help="Enable pdfplumber 'text' strategy (can re-introduce false tables; off by default).",
    )
    ap.add_argument(
        "--max-pages",
        type=int,
        default=0,
        help="Debug: stop after N PDF pages processed (0 = all).",
    )
    return ap.parse_args(argv)


def setup_logging(level: str, log_file: Optional[str], table_debug: bool) -> None:
    lvl = getattr(logging, level.upper(), logging.INFO)
    handlers: List[logging.Handler] = [logging.StreamHandler(sys.stdout)]
    if log_file:
        handlers.append(logging.FileHandler(log_file, encoding="utf-8"))
    logging.basicConfig(
        level=lvl,
        format="%(asctime)s | %(levelname)-7s | %(message)s",
        handlers=handlers,
    )
    if table_debug and lvl > logging.DEBUG:
        logger.setLevel(logging.DEBUG)


def make_single_page_pdf(reader: PdfReader, page_index0: int, tmp_dir: Path) -> Path:
    out_path = tmp_dir / f"page_{page_index0 + 1:04d}.pdf"
    if out_path.exists():
        return out_path
    writer = PdfWriter()
    writer.add_page(reader.pages[page_index0])
    with out_path.open("wb") as f:
        writer.write(f)
    return out_path


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    setup_logging(args.log_level, args.log_file, args.table_debug)

    src_pdf: Path = args.pdf
    out_dir: Path = args.out
    out_dir.mkdir(parents=True, exist_ok=True)

    logger.info("Input PDF : %s", src_pdf)
    logger.info("Out dir   : %s", out_dir)
    logger.info(
        "Camelot   : %s", "available" if camelot is not None else "not installed"
    )
    logger.info("Crop      : top=%.2f bottom=%.2f", args.crop_top, args.crop_bottom)
    logger.info(
        "pdfplumber : text-strategy=%s", "ON" if args.enable_text_strategy else "OFF"
    )

    starts = detect_chapters_from_toc(
        src_pdf, expected=args.chapters, scan_pages=args.toc_scan_pages
    )
    if not starts:
        logger.error(
            "Could not detect chapters from TOC. Try increasing --toc-scan-pages."
        )
        return 2

    reader = PdfReader(str(src_pdf))
    total_pages = len(reader.pages)
    chapters = build_chapter_ranges(starts, total_pages)

    for num, title, start_p in starts:
        logger.info("Chapter %d starts at page %d: %s", num, start_p, title)

    write_index(out_dir, src_pdf, chapters)

    t_all = time.perf_counter()
    processed_pages = 0

    with (
        tempfile.TemporaryDirectory(prefix="pdfpages_") as td,
        pdfplumber.open(str(src_pdf)) as pdf,
    ):
        tmp_dir = Path(td)

        for ch in chapters:
            fname = f"chapter_{ch.number:02d}_{slugify(ch.title)}.md"
            md_path = out_dir / fname

            logger.info("Writing Chapter %d (%s): %s", ch.number, ch.title, md_path)
            with md_path.open("w", encoding="utf-8") as f_out:
                f_out.write(f"# Chapter {ch.number}: {ch.title}\n\n")
                f_out.write(
                    f"> Source pages {ch.start_page_1based}–{ch.end_page_1based} in `{src_pdf.name}`.\n\n"
                )

                for page_num in range(ch.start_page_1based, ch.end_page_1based + 1):
                    if args.max_pages and processed_pages >= args.max_pages:
                        logger.warning(
                            "Stopped early due to --max-pages=%d", args.max_pages
                        )
                        logger.info("All done. Output folder: %s", out_dir)
                        return 0

                    t_page = time.perf_counter()
                    processed_pages += 1

                    page_index0 = page_num - 1
                    page = pdf.pages[page_index0]
                    page_body = crop_header_footer(
                        page, top_crop=args.crop_top, bottom_crop=args.crop_bottom
                    )

                    tmp_page_pdf = make_single_page_pdf(reader, page_index0, tmp_dir)

                    tables = extract_tables_for_page(
                        page_body,
                        tmp_page_pdf,
                        table_debug=args.table_debug,
                        enable_text_strategy=args.enable_text_strategy,
                    )

                    f_out.write(f"\n## Page {page_num}\n\n")

                    bbox_tables = [t for t in tables if t.bbox is not None]
                    other_tables = [t for t in tables if t.bbox is None]

                    if bbox_tables:
                        bboxes = [t.bbox for t in bbox_tables if t.bbox is not None]  # type: ignore[arg-type]
                        text_regions = subtract_table_regions(
                            page_body, table_bboxes=bboxes
                        )

                        bbox_tables_sorted = sorted(
                            bbox_tables,
                            key=lambda t: t.bbox[1] if t.bbox else 0,  # type: ignore[index]
                        )
                        ti = 0

                        for region in text_regions:
                            raw_text = extract_text_in_bbox(page_body, region)
                            page_text = clean_page_text(raw_text)
                            if page_text:
                                f_out.write(page_text + "\n\n")

                            _, _, _, region_y1 = region
                            while ti < len(bbox_tables_sorted):
                                tb = bbox_tables_sorted[ti]
                                if tb.bbox and tb.bbox[1] <= region_y1 + 1:  # type: ignore[index]
                                    f_out.write(grid_to_markdown(tb.grid) + "\n\n")
                                    ti += 1
                                else:
                                    break

                        while ti < len(bbox_tables_sorted):
                            f_out.write(
                                grid_to_markdown(bbox_tables_sorted[ti].grid) + "\n\n"
                            )
                            ti += 1
                    else:
                        raw_text = page_body.extract_text(layout=False) or ""
                        page_text = clean_page_text(raw_text)
                        if page_text:
                            f_out.write(page_text + "\n\n")

                    for tb in other_tables:
                        f_out.write(grid_to_markdown(tb.grid) + "\n\n")

                    if args.table_debug:
                        logger.debug(
                            "Page %d: tables=%d (bbox=%d, other=%d)",
                            page_num,
                            len(tables),
                            len(bbox_tables),
                            len(other_tables),
                        )

                    logger.info(
                        "Done page %d in %.2fs", page_num, time.perf_counter() - t_page
                    )

                    if processed_pages % 25 == 0:
                        gc.collect()

    logger.info("All done. Output folder: %s", out_dir)
    logger.info("Total time: %.2fs", time.perf_counter() - t_all)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
