from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont


ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "frontend" / "public" / "share"
LOGO_PATH = ROOT / "frontend" / "src" / "assets" / "gs-store-system-logo.jpg"

WIDTH = 1200
HEIGHT = 630


def load_font(size: int, bold: bool = False):
    candidates = [
        "C:/Windows/Fonts/msyhbd.ttc" if bold else "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/simhei.ttf" if bold else "C:/Windows/Fonts/simsun.ttc",
    ]
    for path in candidates:
        try:
            return ImageFont.truetype(path, size=size)
        except OSError:
            continue
    return ImageFont.load_default()


def rounded_rect(draw, box, radius, fill):
    draw.rounded_rectangle(box, radius=radius, fill=fill)


def paint_cover(
    filename: str,
    eyebrow: str,
    title: str,
    description: str,
    accent_color: tuple[int, int, int],
    dark_color: tuple[int, int, int],
):
    image = Image.new("RGB", (WIDTH, HEIGHT), (245, 241, 232))
    draw = ImageDraw.Draw(image)

    # layered soft gradients
    rounded_rect(draw, (40, 40, WIDTH - 40, HEIGHT - 40), 36, (255, 253, 248))
    rounded_rect(draw, (70, 70, WIDTH - 70, HEIGHT - 70), 32, (248, 244, 236))

    overlay = Image.new("RGBA", (WIDTH, HEIGHT), (0, 0, 0, 0))
    od = ImageDraw.Draw(overlay)
    od.ellipse((-120, -60, 500, 520), fill=(*accent_color, 36))
    od.ellipse((760, 260, 1320, 860), fill=(*dark_color, 28))
    overlay = overlay.filter(ImageFilter.GaussianBlur(28))
    image = Image.alpha_composite(image.convert("RGBA"), overlay).convert("RGB")
    draw = ImageDraw.Draw(image)

    # content card
    rounded_rect(draw, (96, 108, WIDTH - 96, HEIGHT - 108), 34, (255, 255, 255))
    rounded_rect(draw, (126, 138, WIDTH - 126, HEIGHT - 138), 30, (249, 250, 247))

    # logo
    logo = Image.open(LOGO_PATH).convert("RGB").resize((120, 120))
    mask = Image.new("L", (120, 120), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, 120, 120), radius=28, fill=255)
    image.paste(logo, (156, 168), mask)

    eyebrow_font = load_font(28, bold=True)
    title_font = load_font(74, bold=True)
    desc_font = load_font(28, bold=False)
    tag_font = load_font(24, bold=True)

    draw.text((308, 178), eyebrow, font=eyebrow_font, fill=accent_color)
    draw.text((308, 228), title, font=title_font, fill=dark_color)
    draw.text((156, 414), description, font=desc_font, fill=(91, 103, 116))

    tags = ["微信卡片分享", "高信任服务入口", "迷彩侠"]
    x = 156
    for tag in tags:
      bbox = draw.textbbox((0, 0), tag, font=tag_font)
      w = bbox[2] - bbox[0] + 34
      rounded_rect(draw, (x, 500, x + w, 548), 24, (*accent_color, 0))
      draw.rounded_rectangle((x, 500, x + w, 548), radius=24, outline=accent_color, width=2)
      draw.text((x + 17, 512), tag, font=tag_font, fill=accent_color)
      x += w + 14

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    image.save(OUT_DIR / filename, quality=92)


def main():
    paint_cover(
        filename="veteran-portal-cover.jpg",
        eyebrow="侠伴行",
        title="服务者工作台",
        description="订单处理、培训学习、个人资料维护与实时派单，都在同一个服务者门户里完成。",
        accent_color=(168, 56, 43),
        dark_color=(20, 44, 70),
    )
    paint_cover(
        filename="xiadaojia-cover.jpg",
        eyebrow="侠到家",
        title="家庭到家服务入口",
        description="支持服务预约、地址管理与订单追踪，面向社区家庭提供高信任到家服务体验。",
        accent_color=(176, 88, 38),
        dark_color=(19, 59, 93),
    )


if __name__ == "__main__":
    main()
