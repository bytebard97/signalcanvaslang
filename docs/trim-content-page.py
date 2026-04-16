#!/usr/bin/env python3
"""
Measure the actual content height of the last page in a PDF and report
the ideal page height. Used for the two-pass render workflow:

  Pass 1: Render with a huge content page (500in)
  Pass 2: This script measures actual content, reports ideal height
  Pass 3: Re-render with the ideal height

Usage:
    python3 trim-content-page.py <input.pdf> [--page 3] [--margin 0.5]

Output:
    Prints the ideal page height in inches to stdout.

Requires: pymupdf (fitz)
    pip install pymupdf
"""

import sys
import argparse

try:
    import fitz  # pymupdf
except ImportError:
    print("ERROR: pymupdf not installed. Run: pip install pymupdf", file=sys.stderr)
    sys.exit(1)


def measure_content_height(pdf_path: str, page_num: int, margin_inches: float) -> float:
    """Find the lowest text or image element on the given page and return ideal height in inches."""
    doc = fitz.open(pdf_path)

    if page_num > len(doc):
        print(f"ERROR: PDF has {len(doc)} pages, requested page {page_num}", file=sys.stderr)
        sys.exit(1)

    page = doc[page_num - 1]  # 0-indexed
    page_height_pts = page.rect.height
    page_width_pts = page.rect.width

    max_y = 0.0

    # Method 1: Text blocks — most reliable for finding actual content bottom
    text_dict = page.get_text("dict")
    for block in text_dict.get("blocks", []):
        # Skip blocks that span the full page width (likely backgrounds)
        bbox = block.get("bbox", (0, 0, 0, 0))
        # Only count blocks that have actual text content
        if block.get("type") == 0:  # text block
            for line in block.get("lines", []):
                for span in line.get("spans", []):
                    if span.get("text", "").strip():
                        if bbox[3] > max_y:
                            max_y = bbox[3]
        elif block.get("type") == 1:  # image block
            if bbox[3] > max_y:
                max_y = bbox[3]

    # Method 2: Embedded images
    for img in page.get_images(full=True):
        xref = img[0]
        rects = page.get_image_rects(xref)
        for inst in rects:
            if inst.y1 > max_y:
                max_y = inst.y1

    doc.close()

    if max_y == 0:
        print("WARNING: No content found on page", file=sys.stderr)
        return page_height_pts / 72.0

    # Convert points to inches and add margin
    content_height_inches = max_y / 72.0
    ideal_height = content_height_inches + margin_inches

    # Report stats
    page_height_inches = page_height_pts / 72.0
    wasted = page_height_inches - content_height_inches
    pct = wasted / page_height_inches * 100 if page_height_inches > 0 else 0
    print(f"  Page {page_num}: {page_height_inches:.1f}in total, "
          f"content ends at {content_height_inches:.1f}in, "
          f"{wasted:.1f}in wasted ({pct:.0f}%)",
          file=sys.stderr)
    print(f"  Ideal height: {ideal_height:.1f}in (content + {margin_inches}in margin)",
          file=sys.stderr)

    return ideal_height


def main():
    parser = argparse.ArgumentParser(description="Measure content height of a PDF page")
    parser.add_argument("pdf", help="Input PDF file")
    parser.add_argument("--page", type=int, default=3, help="Page number to measure (default: 3)")
    parser.add_argument("--margin", type=float, default=1.0, help="Bottom margin in inches (default: 1.0)")
    args = parser.parse_args()

    height = measure_content_height(args.pdf, args.page, args.margin)
    # Print just the number to stdout for script consumption
    print(f"{height:.1f}")


if __name__ == "__main__":
    main()
