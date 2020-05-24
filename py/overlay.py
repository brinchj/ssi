#!/usr/bin/env python3

# setup: pip3 install scikit-image
from skimage import io, draw

# read hospitalizations per day from datafile
points = list(map(int, open('hospitalized.txt').readlines()))

# read the forecast image
img = io.imread('forecast.jpg')

# short-hand to draw a line from (x0, y0) to (x1, y1)
def add_line(img, x0, y0, x1, y1):
  cc, rr, val = draw.line_aa(int(y0), int(x0), int(y1), int(x1))
  img[cc, rr] = (255, 0, 0)

fix_x = 1
fix_y = 1

# coordinate for (0, 0) on the plot
start_x = 84
start_y = 405

# the spacing between x-axis points (days)
x_point_width = 3.76

# the spacing between y-axis points (patients)
y_point_width = 0.165

# we have 523 pixels between the 3 plots
# we draw our plot on top of each
for offset_y in [0, 481, 481*2]:
  prev_x = start_x
  prev_y = start_y + offset_y
  for x, p in enumerate(points):
    new_x = start_x + x * x_point_width
    new_y = start_y - fix_y * p * y_point_width + offset_y
    add_line(img, prev_x * fix_x, prev_y, new_x * fix_x, new_y)
    prev_x = new_x
    prev_y = new_y

# save the result
io.imsave('result.jpg', img)
