SAMPLES=$1
DAYS=$2
EPOCH=$3
LAYOUT=$4
OUTDIR=$5
# add -d to deactivate guards
GUARDS=$6
LAYOUT_BASENAME=$(basename $LAYOUT)
if [[ $GUARDS == "-d" ]]; then
	WITH_GUARD=noguard
else
	WITH_GUARD=withguard
fi

ROUTESIM=routesim

# simple sim with guards disabled
$ROUTESIM --in-dir $LAYOUT --epoch $EPOCH -u simple --users $SAMPLES --days $DAYS $GUARDS | sed 's/;/\n/g' > $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH}_${WITH_GUARD}



python3 process_sim.py --in_file $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH}_${WITH_GUARD} --format sync --outname $OUTDIR/processed_simple_${LAYOUT_BASENAME}_${SAMPLES}_${DAYS}_${EPOCH}_${WITH_GUARD} --samples $SAMPLES

rm $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH}_${WITH_GUARD}

