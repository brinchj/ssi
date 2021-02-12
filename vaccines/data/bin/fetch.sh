#!/bin/sh

set -euo pipefail

curl $(curl https://covid19.ssi.dk/overvagningsdata/vaccinationstilslutning|rg 'https://files.ssi.dk/covid19/vaccinationstilslutning/vaccinationstilslutning-[0-9a-z-]+' -o|head -n1) > vaccinationstilslutning.pdf

curl $(curl https://covid19.ssi.dk/overvagningsdata/download-fil-med-overvaagningdata|rg 'https://files.ssi.dk/covid19/overvagning/data/data-epidemiologiske-rapport-[0-9a-z-]+' -o|head -n1) > data-epidemiologiske-rapport.zip

unzip data-epidemiologiske-rapport.zip

./bin/vaccinations.py > vacciner.csv
