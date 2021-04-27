#!/bin/sh

set -euo pipefail

curl $(curl https://covid19.ssi.dk/overvagningsdata/download-fil-med-vaccinationsdata|rg 'https://files.ssi.dk/covid19/vaccinationsdata/zipfil/vaccinationsdata-[0-9a-z-]+' -o|head -n1) > vaccinationsdata.zip

curl $(curl https://covid19.ssi.dk/overvagningsdata/download-fil-med-overvaagningdata|rg 'https://files.ssi.dk/covid19/overvagning/dashboard/overvaagningsdata-[0-9a-z-]+' -o|head -n1) > data-epidemiologiske-rapport.zip

unzip data-epidemiologiske-rapport.zip
unzip vaccinationsdata.zip

if [ -d ArcGIS_dashboards_data ]; then
  cp ArcGIS_dashboards_data/Vaccine_DB/* Vaccine_DB/
  rm -r  ArcGIS_dashboards_data
fi
