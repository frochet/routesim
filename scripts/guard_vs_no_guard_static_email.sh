SAMPLES=$1
DAYS=$2
EPOCH=$3
LAYOUT=$4
OUTDIR=$5
LAYOUT_BASENAME=$(basename $LAYOUT)

ROUTESIM=routesim

$ROUTESIM --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS -d | sed 's/;/\n/g' > $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_withguard
$ROUTESIM --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS | sed 's/;/\n/g' > $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_noguard

$ROUTESIM --timestamps-h ../testfiles/tariq/Work2_time_data.json --sizes-h ../testfiles/tariq/Work2_size_data.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS  -d | sed 's/;/\n/g' > $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard
$ROUTESIM --timestamps-h ../testfiles/tariq/Work2_time_data.json --sizes-h ../testfiles/tariq/Work2_size_data.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS | sed 's/;/\n/g' > $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard

$ROUTESIM --timestamps-h ../testfiles/frochet/frochet_timestamps.json --sizes-h ../testfiles/frochet/frochet_sizes.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS  -d | sed 's/;/\n/g' > $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard
$ROUTESIM --timestamps-h ../testfiles/frochet/frochet_timestamps.json --sizes-h ../testfiles/frochet/frochet_sizes.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS | sed 's/;/\n/g' > $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard


