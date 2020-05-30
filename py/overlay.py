import lib

data = lib.Prediction('reports/20200506/22.png', x_size=110, y_size=2000)
img1 = [data.process(1010, 1064), data.process(1010, 1702), data.process(1010, 2334)]

p = (1034, 2490)
img2 = [
    lib.Prediction('reports/20200520/13.png', x_size=110, y_size=2000).process(*p),
    lib.Prediction('reports/20200520/14.png', x_size=110, y_size=2000).process(*p),
    lib.Prediction('reports/20200520/14.png', x_size=110, y_size=2000).process(*p),
]

lib.save(lib.merge([img1, img2]), 'result.png')
