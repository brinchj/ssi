import lib

main_title = 'Den simulerede sygehusbelastning for alle indlagte (grå baggrund) og de observerede værdier (rød linje)'

days = 112  # [11 march; 01 july[

title1 = 'Prognoser fra 6. maj 2020'
data = lib.Prediction('reports/20200506/22.png', x_size=days, y_size=2000)
img1 = [data.process(1010, 1064), data.process(1010, 1702), data.process(1010, 2334)]

title2 = 'Prognoser fra 13. maj 2020'
p = (1034, 2490)
img2 = [
    lib.Prediction('reports/20200513/13.png', x_size=days, y_size=2000).process(*p),
    lib.Prediction('reports/20200513/14.png', x_size=days, y_size=2000).process(*p),
    lib.Prediction('reports/20200513/15.png', x_size=days, y_size=2000).process(*p),
]

days = 205  # [11 march; 01 october[
title3 = 'Prognoser for fase 3 fra 20. maj 2020\n        (Prognose kun for Region Hovedstaden)'
img3 = [
    lib.Prediction('reports/20200520/28.png', x_size=days, y_size=1500).process(1020, 1435),
    lib.Prediction('reports/20200520/29.png', x_size=days, y_size=1500).process( 990, 1414),
    lib.Prediction('reports/20200520/30.png', x_size=days, y_size=1500).process(1015, 1400),
]

left_explainer = [
    'Alle voksne overholder retningslinjerne',
    'Halvdelen af voksne overholder retningslinjerne',
    'Adfærd som før corona'
]

final = lib.merge(left_explainer, [(title1, img1), (title2, img2), (title3, img3)])
lib.save(lib.add_header(final, main_title), 'result.png')
