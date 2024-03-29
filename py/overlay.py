import lib

main_title = 'To udvalgte prognoser for antal indlagte per dag fra ekspertrapport af 21. Februar 2021'

days = 122  # [8 Feb; 1 June[

title1 = '      Aktivitet som 8. februar uden genåbning og med flere tests (Rref 0.75)'
data = lib.Prediction('reports/20210221/page19-000-fixed.png', x_size=days, y_size=250)
img1 = [data.process(158, 445)]

title2 = 'Samme scenarie men med 6-7% mere smittespredning (Rref 0.80)'
data = lib.Prediction('reports/20210221/page20-000-fixed.png', x_size=days, y_size=250)
img2 = [data.process(158, 445)]


left_explainer = [
  'Antal indlagte per dag'
]

final = lib.merge(left_explainer, [(title1, img1), (title2, img2)])
lib.save(lib.add_header(final, main_title), 'result.png')
