"""The two layouts (OG card and app icon) as HTML/CSS."""

try:
    from .brand import BRAND
except ImportError:  # Allow the module to be imported when run as a script.
    from brand import BRAND


def esc(value):
    return (
        str(value)
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


def rgba(hex_color, alpha):
    raw = hex_color.replace("#", "")
    full = "".join(c + c for c in raw) if len(raw) == 3 else raw
    r, g, b = (int(full[i : i + 2], 16) for i in (0, 2, 4))
    return f"rgba({r}, {g}, {b}, {alpha})"


def base_css(font_base64):
    font = BRAND["font"]
    palette = BRAND["palette"]
    return f"""
@font-face {{
  font-family: '{font["family"]}';
  src: url(data:font/woff2;base64,{font_base64}) format('woff2');
  font-weight: {font["weight"]};
}}
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
html, body {{ background: transparent; }}
body {{ font-family: '{font["family"]}', monospace; -webkit-font-smoothing: antialiased; }}
#stage {{ position: relative; overflow: hidden; background: {palette["bg"]}; }}
"""


def og_template():
    og = BRAND["og"]
    palette = BRAND["palette"]
    footer = " &nbsp;·&nbsp; ".join(esc(item) for item in og["footer"])

    return f"""
<div id="stage" style="width:{og["width"]}px;height:{og["height"]}px;">
  <div style="position:absolute;inset:0;background:
      radial-gradient({og["glow"]["width"]}px {og["glow"]["height"]}px at {og["glow"]["x"]} {og["glow"]["y"]}, {rgba(palette["og_glow"], og["glow"]["alpha"])}, transparent 70%), {palette["bg"]};"></div>
  <div style="position:absolute;inset:0;background-image:
      linear-gradient({rgba(palette["fg"], og["grid"]["alpha"])} 1px, transparent 1px),
      linear-gradient(90deg, {rgba(palette["fg"], og["grid"]["alpha"])} 1px, transparent 1px);
      background-size:{og["grid"]["size"]}px {og["grid"]["size"]}px;"></div>

  <div style="position:absolute;left:64px;right:64px;top:54px;border-top:2px dashed {palette["rule"]};"></div>
  <div style="position:absolute;left:64px;right:64px;bottom:54px;border-bottom:2px dashed {palette["rule"]};"></div>

  <div style="position:absolute;left:88px;top:96px;width:700px;">
    <div style="font-size:104px;font-weight:700;letter-spacing:-.055em;line-height:1;color:{palette["fg"]};">
      <span style="color:{palette["red_bright"]};">{esc(og["wordmark"]["lead"])}</span>{esc(og["wordmark"]["rest"])}
    </div>
    <div style="margin-top:24px;font-size:28px;line-height:1.45;color:{palette["fg_muted"]};white-space:nowrap;">
      <span style="color:{palette["red"]};">{esc(og["prompt"])} </span>{esc(og["tagline"])}
    </div>
  </div>

  <div style="position:absolute;left:88px;top:304px;width:700px;background:{palette["panel"]};
      border:1px solid {palette["panel_border"]};border-left:3px solid {palette["red_bright"]};border-radius:7px;
      padding:24px 28px;font-size:21px;line-height:1.62;">
    <div style="color:{palette["fg"]};white-space:nowrap;">
      <span style="color:{palette["red"]};">{esc(og["prompt"])}</span> {esc(og["command"])}
    </div>
    <div style="color:{palette["red_bright"]};white-space:nowrap;">{esc(og["output"])}</div>
  </div>

  <div style="position:absolute;left:88px;bottom:82px;font-size:15px;letter-spacing:.10em;
      text-transform:uppercase;color:{palette["fg_dim"]};">
    {footer}
  </div>
</div>"""


def icon_template(size, leaf_path):
    icon = BRAND["icon"]
    palette = BRAND["palette"]
    leaf_size = round(size * icon["leaf"]["scale"])
    font_size = round(size * icon["wordmark"]["scale"])

    return f"""
<div id="stage" style="width:{size}px;height:{size}px;display:flex;align-items:center;justify-content:center;">
  <div style="position:absolute;inset:0;background:
      radial-gradient({icon["glow"]["radius"]}% {icon["glow"]["radius"]}% at 50% 50%, {rgba(palette["red"], icon["glow"]["alpha"])}, {rgba(palette["bg"], 0)} 70%), {palette["bg"]};"></div>
  <div style="position:absolute;left:50%;top:50%;transform:translate(-50%,-50%) rotate({icon["leaf"]["rotate"]}deg);">
    <svg width="{leaf_size}" height="{leaf_size}" viewBox="-24 -24 560 560" style="display:block;opacity:{icon["leaf"]["opacity"]};">
      <path fill="{palette["red"]}" d="{leaf_path}"/>
    </svg>
  </div>
  <div style="position:relative;font-size:{font_size}px;font-weight:700;letter-spacing:{icon["wordmark"]["tracking"]};line-height:1;color:{palette["fg"]};">
    <span style="color:{palette["red_bright"]};">{esc(icon["wordmark"]["lead"])}</span>{esc(icon["wordmark"]["rest"])}
  </div>
</div>"""
