#!/bin/sh

set -euo pipefail

curl $(curl https://covid19.ssi.dk/overvagningsdata/download-fil-med-overvaagningdata|rg 'https://files.ssi.dk/covid19/overvagning/data/data-[0-9a-z-]+' -o|head -n1) > data-epidemiologiske-rapport.zip

unzip data-epidemiologiske-rapport.zip


n=$(grep -n 2021-02-05 Newly_admitted_over_time.csv|rg -o '^[0-9]+')
tail -n "+${n}" Newly_admitted_over_time.csv | rg -o '[0-9]+$' > hospitalized.txt
