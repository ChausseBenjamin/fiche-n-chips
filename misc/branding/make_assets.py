"""Generate the OG image and app icons for 1yr.ca.

Run with `uv run make_assets` from `misc/`. The first run needs Chromium for
Playwright: `uv run playwright install chromium`.
"""

import base64
import re
import tempfile
from pathlib import Path

from PIL import Image
from playwright.sync_api import sync_playwright

try:
    from .brand import BRAND
    from .templates import base_css, icon_template, og_template
except ImportError:  # Allow running the file directly.
    from brand import BRAND
    from templates import base_css, icon_template, og_template

here = Path(__file__).resolve().parent
root = here.parents[1]
assets = root / "app/assets"


def read(relative):
    full = root / relative
    if not full.exists():
        raise SystemExit(f"Missing required file: {relative}")
    return full.read_bytes()


def leaf_path_from_favicon():
    svg = read("app/assets/favicon.svg").decode("utf-8")
    match = re.search(r'<path[^>]*\sd="([^"]+)"', svg)
    if not match:
        raise SystemExit(
            "Could not find the leaf path in app/assets/favicon.svg. "
            "Update the extractor in make_assets.py if the favicon structure changed."
        )
    return match.group(1)


def render(page, html, font_base64, file):
    page.set_content(
        f"<!doctype html><html><head><style>{base_css(font_base64)}</style></head>"
        f"<body>{html}</body></html>",
        wait_until="load",
    )
    page.evaluate("async () => { await document.fonts.ready; }")
    page.wait_for_timeout(150)
    page.locator("#stage").screenshot(path=file)


def write_resized(source, target, width, height):
    with Image.open(source) as image:
        image.resize((width, height), Image.Resampling.LANCZOS).save(
            target, format="PNG"
        )
    print(f"wrote {target.relative_to(root)}")


def main():
    font_base64 = base64.b64encode(read(BRAND["font"]["file"])).decode("ascii")
    leaf_path = leaf_path_from_favicon()

    with tempfile.TemporaryDirectory(prefix="fiche-branding-") as tmp:
        tmp = Path(tmp)
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            try:
                # ── OG image ──────────────────────────────────────────────────
                og_render = tmp / "og-image.png"
                og_context = browser.new_context(
                    viewport={
                        "width": BRAND["og"]["width"],
                        "height": BRAND["og"]["height"],
                    },
                    device_scale_factor=2,
                )
                render(
                    og_context.new_page(),
                    og_template(),
                    font_base64,
                    og_render,
                )
                og_context.close()

                write_resized(
                    og_render,
                    assets / "og-image.png",
                    BRAND["og"]["width"],
                    BRAND["og"]["height"],
                )

                # ── App icons ─────────────────────────────────────────────────
                icon_render = tmp / "icon.png"
                icon_context = browser.new_context(
                    viewport={
                        "width": BRAND["icon"]["size"],
                        "height": BRAND["icon"]["size"],
                    },
                    device_scale_factor=1,
                )
                render(
                    icon_context.new_page(),
                    icon_template(BRAND["icon"]["size"], leaf_path),
                    font_base64,
                    icon_render,
                )
                icon_context.close()

                for output in BRAND["icon"]["outputs"]:
                    write_resized(
                        icon_render,
                        assets / output["file"],
                        output["size"],
                        output["size"],
                    )
            finally:
                browser.close()


if __name__ == "__main__":
    main()
