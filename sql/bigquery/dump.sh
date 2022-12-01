
if [[ -z "$DBNAME" ]]; then
    echo "Must provide DBNAME in environment" 1>&2
    exit 1
fi
if [[ -z "$DBHOST" ]]; then
    echo "Must provide DBHOST in environment" 1>&2
    exit 1
fi
if [[ -z "$DBUSER" ]]; then
    echo "Must provide DBUSER in environment" 1>&2
    exit 1
fi
if [[ -z "$GSPATH" ]]; then
    echo "Must provide GSPATH in environment" 1>&2
    exit 1
fi
if [[ -z "$PGPASSWORD" ]]; then
    echo "Must provide PGPASSWORD in environment" 1>&2
    exit 1
fi

for T in `echo "SELECT DISTINCT table_name FROM information_schema.columns WHERE table_schema='public' AND position('_' in table_name) <> 1 ORDER BY 1" | psql $DBNAME -h $DBHOST -U $DBUSER -P format=unaligned -P tuples_only -P fieldsep=\,`; do
  echo $T
  psql $DBNAME -h $DBHOST -U $DBUSER -c "\COPY $T TO '$T.csv' DELIMITER ',' CSV FORCE QUOTE *;"; cat $T.csv | perl -ne 's/\r/\\r/g;print' | gsutil cp - $GSPATH/$T.csv; rm $T.csv
done
