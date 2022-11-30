
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
  psql $DBNAME -h $DBHOST -U $DBUSER -P format=unaligned -P tuples_only -P fieldsep='___SEPARATOR___' -c "SELECT * FROM $T" | perl -ne 's/\t/\\t/g;s/___SEPARATOR___/\t/g;print' | gsutil cp - $GSPATH/$T.csv
done
