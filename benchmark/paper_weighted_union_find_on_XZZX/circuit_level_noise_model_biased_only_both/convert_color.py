

colors = ["#e41a1c", "#377eb8", "#4daf4a", "#ff7f00", "#984ea3"]
def convert(r, g, b):
    c = lambda x: int(255 - (255 - x) * 0.3)
    return c(r), c(g), c(b)
def rgb_to_hex(r, g, b):
    return '%02x%02x%02x' % (r, g, b)

from PIL import ImageColor

for color in colors:
    r, g, b = ImageColor.getcolor(color, "RGB")
    cr, cg, cb = convert(r, g, b)
    print(rgb_to_hex(cr, cg, cb))
