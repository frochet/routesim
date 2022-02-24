SAMPLES=$1
DAYS=$2
EPOCH=$3
LAYOUT=$4
OUTDIR=$5
LAYOUT_BASENAME=$(basename $LAYOUT)

ROUTESIM=routesim

$ROUTESIM --timestamps-h ../testfiles/timestamps.json --sizes-h ../testfiles/sizes.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS -d | sed 's/;/\n/g' > $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_noguard
$ROUTESIM --timestamps-h ../testfiles/timestamps.json --sizes-h ../testfiles/sizes.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS | sed 's/;/\n/g' > $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_withguard

$ROUTESIM --timestamps-h ../testfiles/tariq/Work2_time_data.json --sizes-h ../testfiles/tariq/Work2_size_data.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS  -d | sed 's/;/\n/g' > $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard
$ROUTESIM --timestamps-h ../testfiles/tariq/Work2_time_data.json --sizes-h ../testfiles/tariq/Work2_size_data.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS | sed 's/;/\n/g' > $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard

$ROUTESIM --timestamps-h ../testfiles/frochet/frochet_timestamps.json --sizes-h ../testfiles/frochet/frochet_sizes.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS  -d | sed 's/;/\n/g' > $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard
$ROUTESIM --timestamps-h ../testfiles/frochet/frochet_timestamps.json --sizes-h ../testfiles/frochet/frochet_sizes.json --in-dir $LAYOUT -u email --users $SAMPLES --days $DAYS | sed 's/;/\n/g' > $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard


# Process results?

python3 process_sim.py --in_file $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_withguard --outname $OUTDIR/processed_email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_withguard --samples $SAMPLES --format async &
python3 process_sim.py --in_file $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_noguard --outname $OUTDIR/processed_email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_noguard --samples $SAMPLES --format async &

# Tariq

python3 process_sim.py --in_file $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard --outname $OUTDIR/processed_tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard --samples $SAMPLES --format async &
python3 process_sim.py --in_file $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard --outname $OUTDIR/processed_tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard --samples $SAMPLES --format async &

# Frochet

python3 process_sim.py --in_file $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard --outname $OUTDIR/processed_frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard --samples $SAMPLES --format async &
python3 process_sim.py --in_file $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard --outname $OUTDIR/processed_frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard --samples $SAMPLES --format async

# rm sim files
#rm $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_withguard
#rm $OUTDIR/email_uoe_${SAMPLES}_${DAYS}_${EPOCH}_noguard
#rm $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard
#rm $OUTDIR/tariq_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard
#rm $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_noguard
#rm $OUTDIR/frochet_email_${SAMPLES}_${DAYS}_${EPOCH}_${LAYOUT_BASENAME}_withguard
