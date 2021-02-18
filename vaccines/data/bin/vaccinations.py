#!/usr/bin/env python3

import subprocess
import re


def fix_number(n):
  return n.replace('.', '').replace('-', '0')

def fix_date(d):
  d, m, y = d.split('-')
  return '{}-{}-{}'.format(y, m, d)


def app():
  subprocess.Popen(
      ['gs', '-sDEVICE=txtwrite', '-dNOPAUSE', '-dSAFER', '-sOutputFile=vacciner.txt', 'vaccinationstilslutning.pdf'],
      stdout=subprocess.PIPE
    )

  txt = open('vacciner.txt').read()

  txt = txt[txt.index('rdigvaccineret pr'):]
  for line in txt.split('\n')[2:]:
    groups = re.split(r'\s+', line)[:8]
    if len(groups) not in (7, 8):
      if len(groups) == 1 or groups[0] in ('Page', 'Substituting'):
        continue
      break

    if len(groups) == 8 and len(groups[1]) == 10:
      print('{}; {:>5}; {:>5}'.format(fix_date(groups[1]), fix_number(groups[2]), fix_number(groups[5])))

  return


if __name__ == '__main__':
  app()
