"""Single source of truth for the generated brand assets.

Change a value here, run `uv run make_assets.py`, and the PNGs in app/assets
update.
"""

BRAND = {
    "font": {
        "family": "1yr-mono",
        # Relative to the repository root.
        "file": "app/assets/misc/jetbrainsmono-latin.woff2",
        "weight": "400 700",
    },
    "palette": {
        "bg": "#0d0d0d",
        "panel": "#141414",
        "panel_border": "#2c2c2c",
        "rule": "#343434",
        "fg": "#e0e0e0",
        "fg_muted": "#b8b8b8",
        "fg_dim": "#6f6f6f",
        "red": "#d62828",
        "red_bright": "#ef233c",
        "cream": "#FEDDBB",
        "og_glow": "#d62828",
    },
    "og": {
        "width": 1200,
        "height": 600,
        "grid": {"size": 48, "alpha": 0.032},
        "glow": {
            "width": 720,
            "height": 520,
            "x": "76%",
            "y": "47%",
            "alpha": 0.2,
        },
        "wordmark": {"lead": "1yr", "rest": ".ca"},
        "tagline": "terminal pastebin & url shortener — hosted in canada",
        "prompt": "$",
        "command": 'echo "I Love Timbits" | nc 1yr.ca 443',
        "output": "https://1yr.ca/x7kq",
        "footer": ["365 day retention", "anonymous", "open source"],
    },
    "icon": {
        "size": 1024,
        "glow": {"radius": 60, "alpha": 0.14},
        # The leaf path is read from app/assets/favicon.svg so the two stay in sync.
        "leaf": {"scale": 0.68359375, "opacity": 0.12, "rotate": -5},
        "wordmark": {
            "lead": "1",
            "rest": "yr",
            "scale": 0.322265625,
            "tracking": "-.055em",
        },
        "outputs": [
            {"file": "apple-touch-icon.png", "size": 180},
            {"file": "icon-maskable-512.png", "size": 512},
        ],
    },
}
