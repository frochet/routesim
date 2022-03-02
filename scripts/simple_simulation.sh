SAMPLES=$1
DAYS=$2
EPOCH=$3
LAYOUT=$4
OUTDIR=$5
LAYOUT_BASENAME=$(basename $LAYOUT)

ROUTESIM=routesim

# simple sim with guards disabled
$ROUTESIM --in-dir $LAYOUT --epoch $EPOCH -u simple --users $SAMPLES --days $DAYS -d | sed 's/;/\n/g' > $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH}



python3 process_sim.py --in_file $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH} --format sync --outname $OUTDIR/processed_simple_${SAMPLES}_${DAYS}_${EPOCH}

rm $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH}

