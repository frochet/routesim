SAMPLES=$1
DAYS=$2
EPOCH=$3
OUTDIR=$4
ROUTESIM=routesim

$ROUTESIM --in-dir ../testfiles/single_layout --users $SAMPLES --days $DAYS --epoch $EPOCH -d | sed 's/;/\n/g' > $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH}_noguard
$ROUTESIM --in-dir ../testfiles/single_layout --users $SAMPLES --days $DAYS --epoch $EPOCH | sed 's/;/\n/g' > $OUTDIR/simple_${SAMPLES}_${DAYS}_${EPOCH}_withguard

