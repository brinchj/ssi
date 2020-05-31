import lib

main_title = 'Den simulerede sygehusbelastning for alle indlagte (grå baggrund) og de observerede værdier (rød linje)'

days = 112

title1 = 'Prognoser fra 6. maj 2020'
data = lib.Prediction('reports/20200506/22.png', x_size=days, y_size=2000)
img1 = [data.process(1010, 1064), data.process(1010, 1702), data.process(1010, 2334)]

title2 = 'Prognoser fra 13. maj 2020'
p = (1034, 2490)
img2 = [
    lib.Prediction('reports/20200520/13.png', x_size=days, y_size=2000).process(*p),
    lib.Prediction('reports/20200520/14.png', x_size=days, y_size=2000).process(*p),
    lib.Prediction('reports/20200520/15.png', x_size=days, y_size=2000).process(*p),
]

left_explainer = [
    'Alle voksne overholder retningslinjerne',
    'Halvdelen af voksne overholder retningslinjerne',
    'Adfærd som før corona'
]

final = lib.merge(left_explainer, [(title1, img1), (title2, img2)])
lib.save(lib.add_header(final, main_title), 'result.png')
