#!/bin/bash
FILE=$1
FILENAME=$(basename -- "$FILE")
count_cols() {
	head -1 $FILE | sed 's/[^,]//g' | wc -c
}

for (( c=3 ; c<$(count_cols); c++ ))
do	
	epoch=$((c-3))
	cut -d "," -f 1-3,$c < $FILE > $PWD/epoch${epoch}_${FILENAME}
done
