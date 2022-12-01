if [[ -z "$PROJECT" ]]; then
    echo "Must provide PROJECT in environment" 1>&2
    exit 1
fi
if [[ -z "$DATASET" ]]; then
    echo "Must provide DATASET in environment" 1>&2
    exit 1
fi
for J in *.json; do
  echo "bq mk --table ${PROJECT}:${DATASET}.${J/.json/} $J"
done
