# setup: pip3 install scikit-image
from skimage import io, draw
from PIL import Image, ImageDraw, ImageFont
import numpy as np

# read hospitalizations per day from datafile
POINTS = list(map(int, open('hospitalized.txt').readlines()))


class Plot:
    def __init__(self, img, start_x, start_y, title=None):
        self.img = img
        self.start_x = start_x
        self.start_y = start_y
        self.title = title


class Prediction:
    def __init__(self, path, x_size, y_size):
        self.path = path
        self.img = io.imread(self.path)
        self.x_size = float(x_size)
        self.y_size = float(y_size)
        self.points = list(map(int, open('hospitalized.txt').readlines()))

    def process(self, *search_xy):
        # Scan y axis.
        y_axis = self.scan(search_xy, lambda x, y: (x, y-1))
        y_start = y_axis[0][1]
        y_end = y_axis[-1][1]

        # Scan x axis.
        x_axis = self.scan(search_xy, lambda x, y: (x+1, y))
        x_start = x_axis[0][0]
        x_end = x_axis[-1][0]

        # Calculate pixels on axis.
        y_pixels = abs(y_start - y_end)
        x_pixels = abs(x_start - x_end)

        # Calculate pixel height and width of units.
        y_point_width = y_pixels / self.y_size
        x_point_width = x_pixels / self.x_size

        # Draw the plot.
        self.plot(
            self.points,
            start_x=x_start, start_y=y_start,
            x_point_width=x_point_width, y_point_width=y_point_width)

        # Cut the image to fit the plot.
        crop_start_x = x_start
        crop_start_y = y_start - y_pixels
        crop_end_x = x_start + x_pixels
        crop_end_y = y_start

        def fix(f, nxt):
            count = 0
            while count < 10:
                count += 1
                line, nxt = f(nxt)
                if any(p[0] < 250 for p in line):
                    count = 0
            return nxt

        for _ in range(1):
            crop_start_y = fix(lambda y: (self.img[y, crop_start_x:crop_end_x], y - 1), crop_start_y)
            crop_end_y = fix(lambda y: (self.img[y, crop_start_x:crop_end_x], y + 1), crop_end_y)
            crop_start_x = fix(lambda x: (self.img[crop_start_y:crop_end_y, x], x - 1), crop_start_x)
            crop_end_x = fix(lambda x: (self.img[crop_start_y:crop_end_y, x], x + 1), crop_end_x)

        return Plot(self.img[crop_start_y:crop_end_y, crop_start_x:crop_end_x],
                    x_start - crop_start_x,
                    y_start - crop_start_y)


    def save(self):
        io.imsave('test.png', self.img)

    def valid(self, x, y):
        return 0 <= x < len(self.img[0]) and 0 <= y < len(self.img)

    def scan(self, search_xy, step):
        def white(xy):
            x, y = xy
            return (x, y), self.img[y, x][0] >= 200 and step(x, y) if self.valid(x, y) else None

        def nonwhite(xy):
            x, y = xy
            return (x, y), self.img[y, x][0] < 200 and step(x, y) if self.valid(x, y) else None

        lines = []

        start = self.replicate(white, search_xy)[-1]
        block = self.replicate(nonwhite, start)

        first = None
        second = None

        while block:
            lines.append(block[0])
            startline = self.replicate(white, block[-1])

            l = len(startline)
            if not first:
                first = l
            elif first and not second:
                second = l
            elif second and abs(l - second) / second > .1:
                break

            block = self.replicate(nonwhite, startline[-1])

        return lines


    def replicate(self, f, nxt):
        results = []
        while nxt:
            v, nxt = f(nxt)
            results.append(v)
        return results

    def plot(self, points, start_x, start_y, x_point_width, y_point_width):
        def add_line(img, x0, y0, x1, y1):
            cc, rr, val = draw.line_aa(int(y0), int(x0), int(y1), int(x1))
            img[cc, rr] = (255, 0, 0, 255)

        prev_x = start_x
        prev_y = start_y
        for x, p in enumerate(points):
            new_x = start_x + x * x_point_width
            new_y = start_y -  p * y_point_width
            add_line(self.img, prev_x, prev_y, new_x, new_y)
            prev_x = new_x
            prev_y = new_y


def merge_vertical(*imgs):
    # Warning: this is a bit rough.
    # Calculate the left margin width, i.e. pixels from left to right before we hit something black.
    left = []
    for i in imgs:
        xs = [0]
        for x in range(len(i[0])):
            white = all(p[0] >= 250 for p in i[0:len(i), x])
            if not white:
                break
            xs.append(x)
        left.append(xs[-1])

    # Final image size will fit largest sub-image.
    max_width = max(l + len(i[0]) for l, i in zip(left, imgs))

    # Pad the images.
    padded = []
    for l, i in zip(left, imgs):
        left_pad = max(left) - l
        right_pad = max_width - left_pad - len(i[0])
        color = [(255, 255), (255, 255), (255, 255)]
        padded.append(np.pad(i, ((0, 0), (left_pad, right_pad), (0, 0)), mode='constant', constant_values=color))

    return np.concatenate(padded)

def merge(left_explainer, img_groups):
    titles = [i[0] for i in img_groups]
    groups = [i[1] for i in img_groups]

    def pad_y(img, before, after):
        color = [(255, 255), (255, 255), (255, 255)]
        return np.pad(img, ((before, after), (0, 0), (0, 0)), mode='constant', constant_values=color)

    max_y = max(max(i.start_y for i in group) for group in groups)
    padded_groups = [[pad_y(i.img, max_y - i.start_y, 0) for i in g] for g in groups]

    group_height = max(max(len(i) for i in group) for group in padded_groups)
    padded_groups = [[pad_y(i, 0, group_height - len(i)) for i in g] for g in padded_groups]

    imgs = [merge_vertical(*group) for group in padded_groups]
    imgs = [add_header(img, title) for img, title in zip(imgs, titles)]

    height = max(len(i) for i in imgs)
    plots = np.concatenate([pad_y(i, 0, height - len(i)) for i in imgs], axis=1)

    # Make space for side-header (explainer)
    color = [(255, 255), (255, 255), (255, 255)]
    plots = np.pad(plots, ((0, 0), (50, 0), (0, 0)), mode='constant', constant_values=color)
    for n, text in enumerate(left_explainer):
        img = new_header(50, group_height, text)
        img = img.transpose(Image.ROTATE_90)
        pil_to_img(img, plots, start_xy=(0, n * group_height + 25))

    return plots


def new_header(height, width, text):
    pil_img = Image.new(mode='RGBA', size=(width, height), color=(255, 255, 255))
    font = ImageFont.truetype("fonts/HelveticaNeue Medium.ttf", 18)

    draw = ImageDraw.Draw(pil_img)
    w, h = draw.textsize(text, font=font)
    draw.multiline_text(((width-w)/2, (height-h)/2), text, (0,0,0), font=font)
    return pil_img


def pil_to_img(pil_img, img, start_xy=(0,0)):
    for y in range(pil_img.height):
        for x in range(pil_img.width):
            img[start_xy[1]+y, start_xy[0]+x] = pil_img.getpixel((x, y))


def add_header(img, text):
    width = len(img[0])
    height = 60
    pil_img = new_header(height, width, text)
    img = np.pad(img, ((height, 0), (0, 0), (0, 0)), mode='constant')
    pil_to_img(pil_img, img)
    return img


def save(img, path):
    io.imsave(path, img)
